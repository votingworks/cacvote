import { assert } from '@votingworks/basics';

export class PinLength {
  private constructor(
    private readonly minimum: number,
    private readonly maximum: number
  ) {
    assert(minimum > 0, 'min must be > 0');
    assert(minimum <= maximum, 'min must be <= max');
    assert(Number.isInteger(minimum), 'min must be an integer');
    assert(Number.isInteger(maximum), 'max must be an integer');
  }

  static range(min: number, max: number): PinLength {
    return new PinLength(min, max);
  }

  static exactly(length: number): PinLength {
    return new PinLength(length, length);
  }

  get min(): number {
    return this.minimum;
  }

  get max(): number {
    return this.maximum;
  }

  get isFixed(): boolean {
    return this.minimum === this.maximum;
  }
}
