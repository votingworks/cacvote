import fc from 'fast-check';
import { randomInt } from './random';

test('randomInt()', () => {
  for (let i = 0; i < 1_000; i += 1) {
    expect(randomInt()).toBeGreaterThanOrEqual(0);
    expect(randomInt()).toBeLessThanOrEqual(Number.MAX_SAFE_INTEGER);
  }
});

test('randomInt(number)', () => {
  fc.assert(
    fc.property(fc.integer({ min: 0 }), (max) => {
      const random = randomInt(max);
      expect(random).toBeGreaterThanOrEqual(0);
      expect(random).toBeLessThanOrEqual(max);
      expect(Number.isSafeInteger(random)).toEqual(true);
    })
  );

  expect(randomInt(0)).toEqual(0);
  expect([0, 1]).toContain(randomInt(1));
  expect(() => randomInt(1.1)).toThrow();
  expect(() => randomInt(Number.MAX_SAFE_INTEGER + 1)).toThrow();
  expect(() => randomInt(-1)).toThrow();
});

test('randomInt(number, number)', () => {
  fc.assert(
    fc.property(
      fc.integer().chain((min) => fc.integer({ min }).map((max) => [min, max])),
      ([min, max]) => {
        const random = randomInt(min, max);
        expect(random).toBeGreaterThanOrEqual(min);
        expect(random).toBeLessThanOrEqual(max);
        expect(Number.isSafeInteger(random)).toEqual(true);
      }
    )
  );

  expect(randomInt(0, 0)).toEqual(0);
  expect([0, 1]).toContain(randomInt(0, 1));
  expect(() => randomInt(1, 0)).toThrow();
  expect(() => randomInt(0, 1.1)).toThrow();
  expect(() => randomInt(-1, Number.MAX_SAFE_INTEGER)).toThrow();
  expect(() => randomInt(Number.MIN_SAFE_INTEGER, 1)).toThrow();
});
