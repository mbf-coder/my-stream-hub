//! Monitoring: FFmpeg progress parsing (Phase 1).
//!
//! Progress lines like `frame= 1234 fps= 60 ... time=00:00:41.13 ...`
//! become `EngineStatsPayload` for the `engine-stats` event.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EngineStatsPayload {
    pub frame: u64,
    pub fps: f32,
    pub bitrate_kbps: f32,
    pub elapsed_secs: u64,
}

pub fn parse_progress_line(line: &str) -> Option<EngineStatsPayload> {
    if !line.contains("frame=") || !line.contains("time=") { return None; }
    let frame = field_value(line, "frame=").and_then(|v| v.trim().parse().ok())?;
    let fps = field_value(line, "fps=").and_then(|v| v.trim().parse().ok())?;
    let bitrate_kbps = field_value(line, "bitrate=").map(|v| parse_bitrate(v.trim())).unwrap_or(0.0);
    let elapsed_secs = field_value(line, "time=").map(parse_timestamp).unwrap_or(0);
    Some(EngineStatsPayload { frame, fps, bitrate_kbps, elapsed_secs })
}

fn field_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let start = line.find(key)? + key.len();
    let rest = line[start..].trim_start();
    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
    if end == 0 { return None; }
    Some(rest[..end].trim())
}

fn parse_bitrate(v: &str) -> f32 {
    v.strip_suffix("kbits/s").and_then(|n| n.parse().ok()).unwrap_or(0.0)
}

fn parse_timestamp(v: &str) -> u64 {
    let parts: Vec<&str> = v.split(':').collect();
    if parts.len() != 3 { return 0; }
    let h: u64 = parts[0].parse().unwrap_or(0);
    let m: u64 = parts[1].parse().unwrap_or(0);
    let s: f32 = parts[2].parse().unwrap_or(0.0);
    h * 3600 + m * 60 + s as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_typical_progress_line() {
        let stats = parse_progress_line("frame= 1234 fps= 60 q=28.0 size=N/A time=00:00:41.13 bitrate=1234.5kbits/s speed=2x").unwrap();
        assert_eq!(stats.frame, 1234);
        assert_eq!(stats.elapsed_secs, 41);
    }
    #[test]
    fn ignores_non_progress_lines() {
        assert!(parse_progress_line("ffmpeg version 7.1").is_none());
        assert!(parse_progress_line("").is_none());
    }
    #[test]
    fn tolerates_na_bitrate() {
        let stats = parse_progress_line("frame= 10 fps= 30 q=0.0 size=N/A time=00:00:00.33 bitrate=N/A speed=1x").unwrap();
        assert_eq!(stats.bitrate_kbps, 0.0);
    }
}
