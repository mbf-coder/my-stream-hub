/** Zod mirrors of the Rust validation rules (streaming + destinations + recording). */
import { z } from 'zod';

export const streamConfigSchema = z.object({
  url: z.string().max(2048).refine((u) => /^rtmps?:\/\//i.test(u) && !/\s/.test(u), {
    message: 'scheme must be rtmp:// or rtmps://, no whitespace',
  }),
  encoder: z.string().min(1, 'encoder must not be empty'),
  width: z.number().int().min(160).max(7680),
  height: z.number().int().min(90).max(4320),
  fps: z.number().int().min(1).max(120),
  video_bitrate_kbps: z.number().int().min(100).max(100_000),
  audio_bitrate_kbps: z.number().int().refine((b) => [64, 96, 128, 160, 192, 256, 320].includes(b), {
    message: 'must be one of 64/96/128/160/192/256/320',
  }),
});

export type StreamConfig = z.infer<typeof streamConfigSchema>;

export const destinationSchema = z.object({
  platform: z.enum(['twitch', 'youTube', 'kick', 'custom']),
  label: z.string().min(1).max(64),
  url: z.string().max(2048).refine((u) => /^rtmps?:\/\//i.test(u) && !/\s/.test(u), {
    message: 'scheme must be rtmp:// or rtmps://, no whitespace',
  }),
});

export type DestinationInput = z.infer<typeof destinationSchema>;

export const recordingConfigSchema = z.object({
  file_name: z.string().min(1).max(256),
  format: z.enum(['mkv', 'mp4']),
  tracks: z.enum(['mixed', 'separate']),
  encoder: z.string().min(1),
  video_bitrate_kbps: z.number().int().min(100).max(100_000),
});

export type RecordingConfig = z.infer<typeof recordingConfigSchema>;

/** Redact the trailing path segment (the stream key) for diagnostics. */
export function redactedRtmpUrl(url: string): string {
  const i = url.lastIndexOf('/');
  if (i >= 0 && i + 1 < url.length) return `${url.slice(0, i)}/***`;
  return '***';
}
