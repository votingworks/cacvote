// eslint-disable-next-line max-classes-per-file
import { certs } from '@votingworks/auth';
import { Result } from '@votingworks/basics';
import { NewType, safeParse, safeParseJson } from '@votingworks/types';
import { Buffer } from 'buffer';
import { DateTime } from 'luxon';
import { validate } from 'uuid';
import { ZodError, z } from 'zod';

export type Uuid = NewType<string, 'Uuid'>;

export const UuidSchema = z.string().refine(validate, {
  message: 'Invalid UUID',
}) as unknown as z.ZodSchema<Uuid>;

export type JurisdictionCode = NewType<string, 'JurisdictionCode'>;

export const JurisdictionCodeSchema = z
  .string()
  .refine((s) => /^[a-z]{2}\.[-_a-z0-9]+$/i.test(s), {
    message: 'Invalid jurisdiction code',
  }) as unknown as z.ZodSchema<JurisdictionCode>;

export type JournalEntryAction = 'create' | 'delete' | string;

export const DateTimeSchema = z
  .string()
  .transform(DateTime.fromISO) as unknown as z.ZodSchema<DateTime>;

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

export class Payload {
  constructor(
    private readonly objectType: string,
    private readonly data: Buffer
  ) {}

  getObjectType(): string {
    return this.objectType;
  }

  getData(): Buffer {
    return this.data;
  }

  toJSON(): unknown {
    return {
      objectType: this.objectType,
      data: this.data.toString('base64'),
    };
  }
}

export const PayloadSchema: z.ZodSchema<Payload> = z
  .object({
    objectType: z.string(),
    data: z.string().transform((s) => Buffer.from(s, 'base64')),
  })
  .transform(
    (o) => new Payload(o.objectType, o.data)
  ) as unknown as z.ZodSchema<Payload>;

export class SignedObject {
  constructor(
    private readonly id: Uuid,
    private readonly payload: Buffer,
    private readonly certificates: Buffer,
    private readonly signature: Buffer
  ) {}

  getId(): Uuid {
    return this.id;
  }

  getPayloadRaw(): Buffer {
    return this.payload;
  }

  getPayload(): Result<Payload, ZodError | SyntaxError> {
    return safeParseJson(this.payload.toString(), PayloadSchema);
  }

  getCertificates(): Buffer {
    return this.certificates;
  }

  getSignature(): Buffer {
    return this.signature;
  }

  async getJurisdictionCode(): Promise<Result<JurisdictionCode, ZodError>> {
    const fields = await certs.getCertSubjectFields(this.certificates);
    return safeParse(
      JurisdictionCodeSchema,
      fields.get(certs.VX_CUSTOM_CERT_FIELD.JURISDICTION)
    );
  }

  toJSON(): unknown {
    return {
      id: this.id.toString(),
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

  toJSON(): unknown {
    return {
      id: this.id.toString(),
      objectId: this.objectId.toString(),
      jurisdiction: this.jurisdiction,
      objectType: this.objectType,
      action: this.action,
      createdAt: this.createdAt.toISO(),
    };
  }
}
