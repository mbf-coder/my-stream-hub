import { describe, expect, it } from 'vitest';
import { destinationSchema, recordingConfigSchema } from './stream';

describe('destinationSchema (mirrors Rust Destination::validate_new)', () => {
  it('accepts a valid destination', () => {
    expect(destinationSchema.safeParse({ platform: 'twitch', label: 'Main', url: 'rtmp://live.twitch.tv/app' }).success).toBe(true);
  });
  it('rejects blank labels and non-RTMP urls', () => {
    expect(destinationSchema.safeParse({ platform: 'twitch', label: '', url: 'rtmp://live.twitch.tv/app' }).success).toBe(false);
    expect(destinationSchema.safeParse({ platform: 'custom', label: 'Lab', url: 'https://x.com/live' }).success).toBe(false);
  });
});

describe('recordingConfigSchema (mirrors Rust RecordingConfig)', () => {
  it('accepts mkv/mp4 configs and rejects absurd bitrates', () => {
    const base = { file_name: 'show-01.mkv', format: 'mkv', tracks: 'mixed', encoder: 'libx264', video_bitrate_kbps: 8000 };
    expect(recordingConfigSchema.safeParse(base).success).toBe(true);
    expect(recordingConfigSchema.safeParse({ ...base, format: 'avi' }).success).toBe(false);
    expect(recordingConfigSchema.safeParse({ ...base, video_bitrate_kbps: 5 }).success).toBe(false);
  });
});
