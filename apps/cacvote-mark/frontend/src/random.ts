import { assert } from '@votingworks/basics';

export function randomInt(): number;
export function randomInt(max: number): number;
export function randomInt(min: number, max: number): number;
export function randomInt(min?: number, max?: number): number {
  if (typeof min === 'undefined' && typeof max === 'undefined') {
    return randomInt(0, Number.MAX_SAFE_INTEGER);
  }

  if (typeof min === 'number' && typeof max === 'undefined') {
    return randomInt(0, min);
  }

  assert(typeof min === 'number' && typeof max === 'number');
  assert(min <= max, 'min must be less than or equal to max');
  assert(Number.isSafeInteger(min), 'min must be a safe integer');
  assert(Number.isSafeInteger(max), 'max must be a safe integer');

  if (min === max) {
    return min;
  }

  const range = max - min;
  assert(Number.isSafeInteger(range), 'max - min must be a safe integer');

  const buffer = crypto.getRandomValues(new Uint32Array(2));
  const random = buffer[0] * 0x100000000 + buffer[1];

  return min + (random % (range + 1));
}
