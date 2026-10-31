//! Managed FFmpeg child processes: argv builders, drain pipes, graceful stop, no orphans.

use crate::errors::{EngineError, EngineResult};
use std::collections::VecDeque;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

pub const STDERR_RING_CAP: usize = 200;
pub const DEFAULT_STOP_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub struct FfmpegJob { pub args: Vec<String>, pub stop_timeout: Duration }

impl FfmpegJob {
    pub fn new(args: Vec<String>) -> Self { Self { args, stop_timeout: DEFAULT_STOP_TIMEOUT } }
    pub fn media_test() -> Self {
        Self::new(vec![
            "-hide_banner".into(), "-loglevel".into(), "info".into(),
            "-f".into(), "lavfi".into(), "-i".into(), "testsrc=size=1280x720:rate=30".into(),
            "-f".into(), "lavfi".into(), "-i".into(), "sine=frequency=1000:sample_rate=48000".into(),
            "-t".into(), "3600".into(), "-c:v".into(), "libx264".into(),
            "-preset".into(), "veryfast".into(), "-c:a".into(), "aac".into(),
            "-f".into(), "null".into(), "-".into(),
        ])
    }
}

#[derive(Debug)]
pub struct ManagedProcess {
    child: Child,
    stderr_ring: Arc<Mutex<VecDeque<String>>>,
    cancel: CancellationToken,
    drain_task: Option<tokio::task::JoinHandle<()>>,
    pid: u32,
}

impl ManagedProcess {
    pub fn pid(&self) -> u32 { self.pid }
    pub fn recent_stderr(&self) -> Vec<String> {
        self.stderr_ring.lock().map(|r| r.iter().cloned().collect()).unwrap_or_default()
    }
    pub async fn is_running(&mut self) -> bool { matches!(self.child.try_wait(), Ok(None)) }
    pub async fn stop(mut self, timeout: Duration) -> EngineResult<()> {
        self.cancel.cancel();
        if let Some(mut stdin) = self.child.stdin.take() {
            let _ = tokio::time::timeout(Duration::from_millis(500), stdin.write_all(b"q")).await;
            let _ = stdin.shutdown().await;
        }
        match tokio::time::timeout(timeout, self.child.wait()).await {
            Ok(Ok(status)) => { info!(pid = self.pid, ?status, "managed process stopped"); self.abort_drain(); Ok(()) }
            Ok(Err(e)) => { warn!(pid = self.pid, error = %e, "wait failed; killing"); self.kill().await }
            Err(_) => { warn!(pid = self.pid, "graceful stop timed out; killing"); self.kill().await }
        }
    }
    async fn kill(mut self) -> EngineResult<()> {
        self.child.kill().await.map_err(|e| EngineError::Process(format!("spawn failed: {e}"))).map(|_| ())
            .map_err(|e| EngineError::Process(e.to_string()))?;
        let _ = self.child.wait().await;
        self.abort_drain();
        Ok(())
    }
    fn abort_drain(&mut self) { if let Some(h) = self.drain_task.take() { h.abort(); } }
}

#[derive(Debug, Default)]
pub struct ProcessManager;

impl ProcessManager {
    pub fn new() -> Self { Self }
    pub async fn spawn(&self, job: &FfmpegJob, binary: &str) -> EngineResult<ManagedProcess> {
        if binary != super::ffmpeg::FFMPEG_BIN {
            return Err(EngineError::InvalidParameter("binary".into(), "only the managed ffmpeg binary may be spawned".into()));
        }
        let mut child = Command::new(binary).args(&job.args)
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .kill_on_drop(true).spawn()
            .map_err(|e| EngineError::Process(format!("spawn failed: {e}")))?;
        let pid = child.id().unwrap_or(0);
        let stderr = child.stderr.take();
        let stdout = child.stdout.take();
        let ring: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::with_capacity(STDERR_RING_CAP)));
        let cancel = CancellationToken::new();
        tokio::spawn(async move {
            if let Some(stdout) = stdout {
                use tokio::io::AsyncReadExt;
                let mut reader = BufReader::new(stdout);
                let mut buf = vec![0u8; 8192];
                loop {
                    match reader.read(&mut buf).await { Ok(0) | Err(_) => break, Ok(_) => continue }
                }
            }
        });
        let drain_task = {
            let ring = Arc::clone(&ring);
            let token = cancel.clone();
            tokio::spawn(async move {
                if let Some(stderr) = stderr {
                    let mut lines = BufReader::new(stderr).lines();
                    loop {
                        tokio::select! {
                            _ = token.cancelled() => break,
                            line = lines.next_line() => {
                                match line {
                                    Ok(Some(text)) => {
                                        if let Ok(mut r) = ring.lock() {
                                            if r.len() >= STDERR_RING_CAP { r.pop_front(); }
                                            r.push_back(text);
                                        }
                                    }
                                    _ => break,
                                }
                            }
                        }
                    }
                }
            })
        };
        info!(pid, args = ?job.args, "spawned managed ffmpeg");
        Ok(ManagedProcess { child, stderr_ring: ring, cancel, drain_task: Some(drain_task), pid })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_arbitrary_binaries() {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            let pm = ProcessManager::new();
            let err = pm.spawn(&FfmpegJob::media_test(), "cmd.exe").await.unwrap_err();
            assert!(matches!(err, EngineError::InvalidParameter(_, _)));
        });
    }
    #[test]
    fn media_test_job_writes_to_null_muxer_only() {
        let job = FfmpegJob::media_test();
        assert!(job.args.windows(2).any(|w| w == ["-f", "null"] || w == ["null", "-"]));
        assert!(!job.args.iter().any(|a| a.starts_with("rtmp")));
    }
}
