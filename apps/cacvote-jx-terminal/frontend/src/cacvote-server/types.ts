// TODO: much of this is duplicated. Refactor to use a single source of truth
// for the types.

import { NewType } from '@votingworks/types';
import { Buffer } from 'buffer';
import { DateTime } from 'luxon';
import { v4, validate } from 'uuid';
import { z } from 'zod';

export type Uuid = NewType<string, 'Uuid'>;

export function Uuid(): Uuid {
  return v4() as Uuid;
}

export const UuidSchema = z.string().refine(validate, {
  message: 'Invalid UUID',
}) as unknown as z.ZodSchema<Uuid>;

export type JurisdictionCode = NewType<string, 'JurisdictionCode'>;

export const JurisdictionCodeSchema = z
  .string()
  .refine((s) => /^[a-z]{2}\.[-_a-z0-9]+$/i.test(s), {
    message: 'Invalid jurisdiction code',
  }) as unknown as z.ZodSchema<JurisdictionCode>;

export type Base64String = NewType<string, 'Base64String'>;

export const Base64StringSchema: z.ZodSchema<Buffer> = z
  .string()
  .transform((s) => Buffer.from(s, 'base64')) as unknown as z.ZodSchema<Buffer>;

export const Iso8601DateSchema: z.ZodSchema<DateTime> = z
  .string()
  .transform((s) => DateTime.fromISO(s)) as unknown as z.ZodSchema<DateTime>;
