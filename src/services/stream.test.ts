import { describe, expect, it } from 'vitest';
import { redactedRtmpUrl, streamConfigSchema } from './stream';

const valid = {
  url: 'rtmp://live.twitch.tv/app',
  encoder: 'libx264',
  width: 1920,
  height: 1080,
  fps: 60,
  video_bitrate_kbps: 6000,
  audio_bitrate_kbps: 160,
};

describe('streamConfigSchema (mirrors Rust StreamConfig::validate)', () => {
  it('accepts a valid config', () => {
    expect(streamConfigSchema.safeParse(valid).success).toBe(true);
  });
  it('rejects non-RTMP schemes (incl. shell-injection shapes)', () => {
    expect(streamConfigSchema.safeParse({ ...valid, url: 'https://example.com/live' }).success).toBe(false);
    expect(streamConfigSchema.safeParse({ ...valid, url: 'rtmp://x; rm -rf /' }).success).toBe(false);
  });
  it('rejects absurd fps / bitrate', () => {
    expect(streamConfigSchema.safeParse({ ...valid, fps: 999 }).success).toBe(false);
    expect(streamConfigSchema.safeParse({ ...valid, video_bitrate_kbps: 5 }).success).toBe(false);
  });
});

describe('redactedRtmpUrl', () => {
  it('hides the key segment', () => {
    expect(redactedRtmpUrl('rtmp://live.twitch.tv/app/live_abc123')).toBe('rtmp://live.twitch.tv/app/***');
  });
});
