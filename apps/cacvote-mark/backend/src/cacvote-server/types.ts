// eslint-disable-next-line max-classes-per-file
import { certs } from '@votingworks/auth';
import { Optional, Result, resultBlock } from '@votingworks/basics';
import {
  BallotStyleId,
  BallotStyleIdSchema,
  NewType,
  PrecinctId,
  PrecinctIdSchema,
  safeParse,
  safeParseJson,
} from '@votingworks/types';
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

export const RawJournalEntrySchema: z.ZodSchema<{
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

export const JournalEntrySchema: z.ZodSchema<JournalEntry> =
  RawJournalEntrySchema.transform(
    (o) =>
      new JournalEntry(
        o.id,
        o.objectId,
        o.jurisdiction,
        o.objectType,
        o.action,
        o.createdAt
      )
  ) as unknown as z.ZodSchema<JournalEntry>;

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

  toBuffer(): Buffer {
    return Buffer.from(JSON.stringify(this));
  }

  static of(objectType: string, serializable: unknown): Payload {
    return new Payload(objectType, Buffer.from(JSON.stringify(serializable)));
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

  parsePayloadAs<T>(
    expectedObjectType: string,
    schema: z.ZodSchema<T>
  ): Result<Optional<T>, ZodError | SyntaxError> {
    return resultBlock((bail) => {
      const payload = this.getPayload().okOrElse(bail);
      if (payload.getObjectType() !== expectedObjectType) {
        return undefined;
      }
      const jsonData = payload.getData().toString('utf-8');
      return safeParseJson(jsonData, schema);
    });
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

export const SignedObjectSchema: z.ZodSchema<SignedObject> = z
  .object({
    id: UuidSchema,
    payload: z.instanceof(Buffer),
    certificates: z.instanceof(Buffer),
    signature: z.instanceof(Buffer),
  })
  .transform(
    (o) => new SignedObject(o.id, o.payload, o.certificates, o.signature)
  ) as unknown as z.ZodSchema<SignedObject>;

export class RegistrationRequest {
  constructor(
    private readonly commonAccessCardId: string,
    private readonly jurisdiction: JurisdictionCode,
    private readonly givenName: string,
    private readonly familyName: string,
    private readonly createdAt: DateTime
  ) {}

  getCommonAccessCardId(): string {
    return this.commonAccessCardId;
  }

  getJurisdiction(): JurisdictionCode {
    return this.jurisdiction;
  }

  getGivenName(): string {
    return this.givenName;
  }

  getFamilyName(): string {
    return this.familyName;
  }

  getCreatedAt(): DateTime {
    return this.createdAt;
  }

  toJSON(): unknown {
    return {
      commonAccessCardId: this.commonAccessCardId,
      jurisdiction: this.jurisdiction,
      givenName: this.givenName,
      familyName: this.familyName,
      createdAt: this.createdAt.toISO(),
    };
  }
}

export const RegistrationRequestSchema: z.ZodSchema<RegistrationRequest> = z
  .object({
    commonAccessCardId: z.string(),
    jurisdiction: JurisdictionCodeSchema,
    givenName: z.string(),
    familyName: z.string(),
    createdAt: DateTimeSchema,
  })
  .transform(
    (o) =>
      new RegistrationRequest(
        o.commonAccessCardId,
        o.jurisdiction,
        o.givenName,
        o.familyName,
        o.createdAt
      )
  ) as unknown as z.ZodSchema<RegistrationRequest>;

export class Registration {
  constructor(
    private readonly commonAccessCardId: string,
    private readonly jurisdiction: JurisdictionCode,
    private readonly registrationRequestObjectId: Uuid,
    private readonly electionObjectId: Uuid,
    private readonly ballotStyleId: BallotStyleId,
    private readonly precinctId: PrecinctId
  ) {}

  getCommonAccessCardId(): string {
    return this.commonAccessCardId;
  }

  getJurisdiction(): JurisdictionCode {
    return this.jurisdiction;
  }

  getRegistrationRequestObjectId(): Uuid {
    return this.registrationRequestObjectId;
  }

  getElectionObjectId(): Uuid {
    return this.electionObjectId;
  }

  getBallotStyleId(): BallotStyleId {
    return this.ballotStyleId;
  }

  getPrecinctId(): PrecinctId {
    return this.precinctId;
  }
}

export const RegistrationSchema: z.ZodSchema<Registration> = z
  .object({
    commonAccessCardId: z.string(),
    jurisdiction: JurisdictionCodeSchema,
    registrationRequestObjectId: UuidSchema,
    electionObjectId: UuidSchema,
    ballotStyleId: BallotStyleIdSchema,
    precinctId: PrecinctIdSchema,
  })
  .transform(
    (o) =>
      new Registration(
        o.commonAccessCardId,
        o.jurisdiction,
        o.registrationRequestObjectId,
        o.electionObjectId,
        o.ballotStyleId,
        o.precinctId
      )
  ) as unknown as z.ZodSchema<Registration>;
