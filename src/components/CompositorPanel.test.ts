import { describe, expect, it } from 'vitest';
import { defaultLayer, defaultTransform, sanitizeLayer } from './CompositorPanel';

describe('defaultTransform / defaultLayer', () => {
  it('produces a full-HD identity transform for a given input', () => {
    expect(defaultLayer(2)).toEqual({
      input_index: 2,
      transform: { x: 0, y: 0, width: 1920, height: 1080, rotation_deg: 0, crop: [0, 0, 0, 0] },
    });
    expect(defaultTransform().crop).toEqual([0, 0, 0, 0]);
  });
});

describe('sanitizeLayer (mirrors Rust u32/i32 bounds before IPC)', () => {
  it('floors to integers and clamps unsigned fields at >= 1 where required', () => {
    const out = sanitizeLayer({
      input_index: 1.9,
      transform: { x: -3.7, y: 10.2, width: 0, height: -5, rotation_deg: 45.9, crop: [1.5, -2, 3, 4] },
    });
    expect(out.input_index).toBe(1);
    expect(out.transform.x).toBe(-3);
    expect(out.transform.width).toBe(1);
    expect(out.transform.height).toBe(1);
    expect(out.transform.rotation_deg).toBe(45);
    expect(out.transform.crop).toEqual([1, 0, 3, 4]);
  });

  it('maps NaN to safe zeroes', () => {
    const out = sanitizeLayer({
      input_index: NaN,
      transform: { x: NaN, y: NaN, width: NaN, height: NaN, rotation_deg: NaN, crop: [NaN, NaN, NaN, NaN] },
    });
    expect(out).toEqual({
      input_index: 0,
      transform: { x: 0, y: 0, width: 1, height: 1, rotation_deg: 0, crop: [0, 0, 0, 0] },
    });
  });
});
