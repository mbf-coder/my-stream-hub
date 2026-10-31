//! Scenes, sources and transitions — Phase 2 Studio rundown state.

use crate::errors::{EngineError, EngineResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

pub const MAX_SCENES: usize = 32;
pub const MAX_SOURCES_PER_SCENE: usize = 24;
pub const MAX_NAME_LEN: usize = 64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    #[default]
    Display,
    Window,
    Camera,
    Image,
    Text,
    Media,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Source { pub id: String, pub name: String, pub kind: SourceKind, pub visible: bool, pub volume_db: f32 }

impl Source {
    pub fn new(id: String, name: String, kind: SourceKind) -> EngineResult<Self> {
        validate_name(&name)?;
        Ok(Self { id, name: name.trim().to_string(), kind, visible: true, volume_db: 0.0 })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Scene { pub id: String, pub name: String, pub sources: Vec<Source> }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TransitionKind { #[default] Cut, Fade }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transition { pub kind: TransitionKind, pub duration_ms: u32 }

impl Default for Transition {
    fn default() -> Self { Self { kind: TransitionKind::Cut, duration_ms: 300 } }
}

impl Transition {
    pub fn validate(&self) -> EngineResult<()> {
        if self.kind == TransitionKind::Fade && !(50..=5000).contains(&self.duration_ms) {
            return Err(EngineError::InvalidParameter("duration_ms".into(), "fade must be 50..=5000 ms".into()));
        }
        Ok(())
    }
}

fn validate_name(name: &str) -> EngineResult<()> {
    let t = name.trim();
    if t.is_empty() { return Err(EngineError::InvalidParameter("name".into(), "name must not be empty".into())); }
    if t.len() > MAX_NAME_LEN { return Err(EngineError::InvalidParameter("name".into(), format!("name must be <={MAX_NAME_LEN} chars"))); }
    Ok(())
}

#[derive(Debug, Default)]
struct Inner { scenes: Vec<Scene>, preview_id: Option<String>, program_id: Option<String>, transition: Transition, next_scene: u32, next_source: u32 }

#[derive(Debug, Clone, Default)]
pub struct SceneManager { inner: Arc<RwLock<Inner>> }

impl SceneManager {
    pub fn new() -> Self { Self::default() }
    fn find_mut<'a>(scenes: &'a mut [Scene], id: &str) -> EngineResult<&'a mut Scene> {
        scenes.iter_mut().find(|s| s.id == id).ok_or_else(|| EngineError::InvalidParameter("scene_id".into(), "scene not found".into()))
    }
    pub async fn list_scenes(&self) -> Vec<Scene> { self.inner.read().await.scenes.clone() }
    pub async fn preview_id(&self) -> Option<String> { self.inner.read().await.preview_id.clone() }
    pub async fn program_id(&self) -> Option<String> { self.inner.read().await.program_id.clone() }
    pub async fn transition(&self) -> Transition { self.inner.read().await.transition.clone() }
    pub async fn set_transition(&self, t: Transition) -> EngineResult<Transition> {
        t.validate()?;
        self.inner.write().await.transition = t.clone();
        Ok(t)
    }
    pub async fn create_scene(&self, name: String) -> EngineResult<Scene> {
        validate_name(&name)?;
        let mut g = self.inner.write().await;
        if g.scenes.len() >= MAX_SCENES { return Err(EngineError::InvalidParameter("scene".into(), format!("at most {MAX_SCENES} scenes"))); }
        g.next_scene += 1;
        let scene = Scene { id: format!("scene-{}", g.next_scene), name: name.trim().to_string(), sources: Vec::new() };
        if g.preview_id.is_none() { g.preview_id = Some(scene.id.clone()); }
        if g.program_id.is_none() { g.program_id = Some(scene.id.clone()); }
        g.scenes.push(scene.clone());
        Ok(scene)
    }
    pub async fn delete_scene(&self, id: &str) -> EngineResult<()> {
        let mut g = self.inner.write().await;
        let pos = g.scenes.iter().position(|s| s.id == id).ok_or_else(|| EngineError::InvalidParameter("scene_id".into(), "scene not found".into()))?;
        g.scenes.remove(pos);
        let fallback = g.scenes.first().map(|s| s.id.clone());
        if g.preview_id.as_deref() == Some(id) { g.preview_id = fallback.clone(); }
        if g.program_id.as_deref() == Some(id) { g.program_id = fallback; }
        Ok(())
    }
    pub async fn rename_scene(&self, id: &str, name: String) -> EngineResult<Scene> {
        validate_name(&name)?;
        let mut g = self.inner.write().await;
        let scene = Self::find_mut(&mut g.scenes, id)?;
        scene.name = name.trim().to_string();
        Ok(scene.clone())
    }
    pub async fn add_source(&self, scene_id: &str, name: String, kind: SourceKind) -> EngineResult<Source> {
        let mut g = self.inner.write().await;
        g.next_source += 1;
        let source_id = format!("src-{}", g.next_source);
        let scene = Self::find_mut(&mut g.scenes, scene_id)?;
        if scene.sources.len() >= MAX_SOURCES_PER_SCENE { return Err(EngineError::InvalidParameter("source".into(), format!("at most {MAX_SOURCES_PER_SCENE} sources per scene"))); }
        let source = Source::new(source_id, name, kind)?;
        scene.sources.push(source.clone());
        Ok(source)
    }
    pub async fn remove_source(&self, scene_id: &str, source_id: &str) -> EngineResult<()> {
        let mut g = self.inner.write().await;
        let scene = Self::find_mut(&mut g.scenes, scene_id)?;
        let pos = scene.sources.iter().position(|s| s.id == source_id).ok_or_else(|| EngineError::InvalidParameter("source_id".into(), "source not found".into()))?;
        scene.sources.remove(pos);
        Ok(())
    }
    pub async fn toggle_source(&self, scene_id: &str, source_id: &str) -> EngineResult<bool> {
        let mut g = self.inner.write().await;
        let scene = Self::find_mut(&mut g.scenes, scene_id)?;
        let src = scene.sources.iter_mut().find(|s| s.id == source_id).ok_or_else(|| EngineError::InvalidParameter("source_id".into(), "source not found".into()))?;
        src.visible = !src.visible;
        Ok(src.visible)
    }
    pub async fn move_source(&self, scene_id: &str, source_id: &str, to_index: usize) -> EngineResult<Vec<Source>> {
        let mut g = self.inner.write().await;
        let scene = Self::find_mut(&mut g.scenes, scene_id)?;
        let from = scene.sources.iter().position(|s| s.id == source_id).ok_or_else(|| EngineError::InvalidParameter("source_id".into(), "source not found".into()))?;
        let src = scene.sources.remove(from);
        let to = to_index.min(scene.sources.len());
        scene.sources.insert(to, src);
        Ok(scene.sources.clone())
    }
    pub async fn set_preview(&self, id: &str) -> EngineResult<()> {
        let mut g = self.inner.write().await;
        if !g.scenes.iter().any(|s| s.id == id) { return Err(EngineError::InvalidParameter("scene_id".into(), "scene not found".into())); }
        g.preview_id = Some(id.to_string());
        Ok(())
    }
    pub async fn set_program(&self, id: &str) -> EngineResult<()> {
        let mut g = self.inner.write().await;
        if !g.scenes.iter().any(|s| s.id == id) { return Err(EngineError::InvalidParameter("scene_id".into(), "scene not found".into())); }
        g.program_id = Some(id.to_string());
        Ok(())
    }
    pub async fn take_transition(&self) -> EngineResult<String> {
        let mut g = self.inner.write().await;
        let preview = g.preview_id.clone().ok_or_else(|| EngineError::InvalidParameter("preview".into(), "no preview scene selected".into()))?;
        g.transition.validate()?;
        g.program_id = Some(preview.clone());
        Ok(preview)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    async fn manager_with_scene() -> (SceneManager, Scene) {
        let m = SceneManager::new();
        let s = m.create_scene("Main".into()).await.unwrap();
        (m, s)
    }
    #[tokio::test]
    async fn scene_crud_and_buses() {
        let (m, s) = manager_with_scene().await;
        assert_eq!(m.preview_id().await.as_deref(), Some(s.id.as_str()));
        let renamed = m.rename_scene(&s.id, "Intro".into()).await.unwrap();
        assert_eq!(renamed.name, "Intro");
        assert!(m.rename_scene(&s.id, "  ".into()).await.is_err());
        m.delete_scene(&s.id).await.unwrap();
        assert!(m.list_scenes().await.is_empty());
        assert!(m.delete_scene(&s.id).await.is_err());
    }
    #[tokio::test]
    async fn source_layering() {
        let (m, s) = manager_with_scene().await;
        let a = m.add_source(&s.id, "Cam".into(), SourceKind::Camera).await.unwrap();
        let b = m.add_source(&s.id, "Overlay".into(), SourceKind::Image).await.unwrap();
        assert!(!m.toggle_source(&s.id, &a.id).await.unwrap());
        let order = m.move_source(&s.id, &b.id, 0).await.unwrap();
        assert_eq!(order[0].id, b.id);
        m.remove_source(&s.id, &a.id).await.unwrap();
        assert!(m.remove_source(&s.id, &a.id).await.is_err());
    }
    #[tokio::test]
    async fn take_copies_preview_to_program() {
        let m = SceneManager::new();
        let a = m.create_scene("A".into()).await.unwrap();
        let b = m.create_scene("B".into()).await.unwrap();
        m.set_preview(&b.id).await.unwrap();
        assert_eq!(m.program_id().await.as_deref(), Some(a.id.as_str()));
        let live = m.take_transition().await.unwrap();
        assert_eq!(live, b.id);
        assert_eq!(m.program_id().await.as_deref(), Some(b.id.as_str()));
    }
    #[tokio::test]
    async fn fade_duration_is_bounded() {
        let m = SceneManager::new();
        let bad = Transition { kind: TransitionKind::Fade, duration_ms: 99999 };
        assert!(m.set_transition(bad).await.is_err());
        let ok = Transition { kind: TransitionKind::Fade, duration_ms: 500 };
        assert!(m.set_transition(ok).await.is_ok());
    }
}
