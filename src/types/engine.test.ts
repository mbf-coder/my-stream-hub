import { describe, expect, it } from 'vitest';
import { IPC_COMMANDS } from './engine';

/** Frontend must invoke exactly the 49 Phase 1–7 commands from the spec. */
describe('IPC contract', () => {
  it('declares the 49 spec commands with no duplicates', () => {
    expect(IPC_COMMANDS).toHaveLength(49);
    expect(new Set(IPC_COMMANDS).size).toBe(49);
  });
  it('keeps the Phase-1 core intact', () => {
    expect([...IPC_COMMANDS].slice(0, 8)).toEqual([
      'get_engine_status', 'initialize_engine', 'get_media_capabilities',
      'get_available_encoders', 'get_audio_devices', 'get_video_devices',
      'start_media_test', 'stop_media_test',
    ]);
  });
});
