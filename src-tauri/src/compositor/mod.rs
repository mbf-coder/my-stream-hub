//! Compositor model — Phase 3: canvas, transforms, validated `filter_complex` builder.

use crate::errors::{EngineError, EngineResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CanvasPreset {
    #[default]
    P1080,
    P720,
    P4k,
}

impl CanvasPreset {
    pub fn dimensions(self) -> (u32, u32) {
        match self { CanvasPreset::P720 => (1280, 720), CanvasPreset::P1080 => (1920, 1080), CanvasPreset::P4k => (3840, 2160) }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transform {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub rotation_deg: i16,
    pub crop: (u32, u32, u32, u32),
}

impl Default for Transform {
    fn default() -> Self {
        Self { x: 0, y: 0, width: 1920, height: 1080, rotation_deg: 0, crop: (0, 0, 0, 0) }
    }
}

impl Transform {
    pub fn validate(&self, canvas: (u32, u32)) -> EngineResult<()> {
        if !(1..=7680).contains(&self.width) || !(1..=4320).contains(&self.height) {
            return Err(EngineError::InvalidParameter("transform".into(), "layer size must be 1..=7680 x 1..=4320".into()));
        }
        if !(-180..=180).contains(&self.rotation_deg) {
            return Err(EngineError::InvalidParameter("rotation_deg".into(), "must be -180..=180".into()));
        }
        let (cw, ch) = (canvas.0 as i32, canvas.1 as i32);
        if self.x < -cw || self.x > 2 * cw || self.y < -ch || self.y > 2 * ch {
            return Err(EngineError::InvalidParameter("transform".into(), "layer position is absurdly far off-canvas".into()));
        }
        Ok(())
    }
    pub fn filter_fragment(&self) -> String {
        let mut parts = vec![format!("scale={}:{}", self.width, self.height)];
        if self.rotation_deg != 0 { parts.push(format!("rotate={}*PI/180", self.rotation_deg)); }
        let (l, t, r, b) = self.crop;
        if l + r + t + b > 0 {
            let cw = self.width.saturating_sub(l + r).max(1);
            let ch = self.height.saturating_sub(t + b).max(1);
            parts.push(format!("crop={cw}:{ch}:{l}:{t}"));
        }
        parts.join(",")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Layer { pub input_index: usize, pub transform: Transform }

pub fn build_filter_complex(canvas: CanvasPreset, layers: &[Layer]) -> EngineResult<String> {
    if layers.is_empty() { return Err(EngineError::InvalidParameter("layers".into(), "at least one layer required".into())); }
    if layers.len() > 16 { return Err(EngineError::InvalidParameter("layers".into(), "at most 16 layers".into())); }
    let (cw, ch) = canvas.dimensions();
    let mut chain: Vec<String> = Vec::new();
    chain.push(format!("[{}:v]scale={cw}:{ch},setsar=1[base]", layers[0].input_index));
    let mut prev = "base".to_string();
    for (i, layer) in layers.iter().enumerate().skip(1) {
        layer.transform.validate((cw, ch))?;
        let scaled = format!("l{i}");
        chain.push(format!("[{}:v]{},format=rgba[{scaled}]", layer.input_index, layer.transform.filter_fragment()));
        let out = format!("m{i}");
        chain.push(format!("[{prev}][{scaled}]overlay={}:{}[{out}]", layer.transform.x, layer.transform.y));
        prev = out;
    }
    chain.push(format!("[{prev}]format=yuv420p[outv]"));
    Ok(chain.join(";"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn presets_have_expected_dimensions() {
        assert_eq!(CanvasPreset::P720.dimensions(), (1280, 720));
        assert_eq!(CanvasPreset::P1080.dimensions(), (1920, 1080));
        assert_eq!(CanvasPreset::P4k.dimensions(), (3840, 2160));
    }
    #[test]
    fn single_layer_yields_canvas_base() {
        let fc = build_filter_complex(CanvasPreset::P1080, &[Layer { input_index: 0, transform: Transform::default() }]).unwrap();
        assert!(fc.contains("[0:v]scale=1920:1080"));
        assert!(fc.ends_with("[outv]"));
        assert!(!fc.contains("overlay"));
    }
    #[test]
    fn two_layers_produce_overlay_chain() {
        let pip = Transform { x: 1400, y: 800, width: 480, height: 270, ..Transform::default() };
        let fc = build_filter_complex(CanvasPreset::P1080, &[
            Layer { input_index: 0, transform: Transform::default() },
            Layer { input_index: 1, transform: pip },
        ]).unwrap();
        assert!(fc.contains("[1:v]scale=480:270"));
        assert!(fc.contains("overlay=1400:800"));
    }
    #[test]
    fn invalid_transform_is_rejected() {
        let bad = Transform { rotation_deg: 270, ..Transform::default() };
        assert!(bad.validate((1920, 1080)).is_err());
        let layers = vec![
            Layer { input_index: 0, transform: Transform::default() },
            Layer { input_index: 1, transform: bad },
        ];
        assert!(build_filter_complex(CanvasPreset::P1080, &layers).is_err());
        assert!(build_filter_complex(CanvasPreset::P1080, &[]).is_err());
    }
}
