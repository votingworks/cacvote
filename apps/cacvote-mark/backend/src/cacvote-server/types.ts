// eslint-disable-next-line max-classes-per-file
import { Buffer } from 'buffer';
import { NewType } from '@votingworks/types';
import { validate } from 'uuid';
import { DateTime } from 'luxon';
import { z } from 'zod';

export type Uuid = NewType<string, 'Uuid'>;

export const UuidSchema = z.string().refine(validate, {
  message: 'Invalid UUID',
}) as unknown as z.ZodSchema<Uuid>;

export const DateTimeSchema = z
  .string()
  .transform(DateTime.fromISO) as unknown as z.ZodSchema<DateTime>;

export class SignedObject {
  constructor(
    private readonly payload: Buffer,
    private readonly certificates: Buffer,
    private readonly signature: Buffer
  ) {}

  getPayload(): Buffer {
    return this.payload;
  }

  getCertificates(): Buffer {
    return this.certificates;
  }

  getSignature(): Buffer {
    return this.signature;
  }

  toJSON(): unknown {
    return {
      payload: this.payload.toString('base64'),
      certificates: this.certificates.toString('base64'),
      signature: this.signature.toString('base64'),
    };
  }
}

export class JournalEntry {
  constructor(
    private readonly id: Uuid,
    private readonly objectId: Uuid,
    private readonly jurisdiction: JurisdictionCode,
    private readonly objectType: string,
    private readonly action: JournalEntryAction,
    private readonly createdAt: DateTime
  ) {}

  getId(): Uuid {
    return this.id;
  }

  getObjectId(): Uuid {
    return this.objectId;
  }

  getJurisdiction(): JurisdictionCode {
    return this.jurisdiction;
  }

  getObjectType(): string {
    return this.objectType;
  }

  getAction(): JournalEntryAction {
    return this.action;
  }

  getCreatedAt(): DateTime {
    return this.createdAt;
  }
}

export type JurisdictionCode = NewType<string, 'JurisdictionCode'>;

export const JurisdictionCodeSchema = z
  .string()
  .refine((s) => /^[a-z]{2}\.[-_a-z0-9]+$/i.test(s), {
    message: 'Invalid jurisdiction code',
  }) as unknown as z.ZodSchema<JurisdictionCode>;

export type JournalEntryAction = 'create' | 'delete' | string;

export const JournalEntrySchema: z.ZodSchema<{
  id: Uuid;
  objectId: Uuid;
  jurisdiction: JurisdictionCode;
  objectType: string;
  action: string;
  createdAt: DateTime;
}> = z.object({
  id: UuidSchema,
  objectId: UuidSchema,
  jurisdiction: JurisdictionCodeSchema,
  objectType: z.string(),
  action: z.string(),
  createdAt: DateTimeSchema,
});
