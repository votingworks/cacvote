// eslint-disable-next-line max-classes-per-file
import { certs, cryptography } from '@votingworks/auth';
import {
  Result,
  ok,
  throwIllegalValue,
  wrapException,
} from '@votingworks/basics';
import {
  BallotStyleId,
  BallotStyleIdSchema,
  CVR,
  ElectionDefinition,
  NewType,
  PrecinctId,
  PrecinctIdSchema,
  safeParse,
  safeParseElectionDefinition,
  safeParseJson,
} from '@votingworks/types';
import { Buffer } from 'buffer';
import { DateTime } from 'luxon';
import { v4, validate } from 'uuid';
import { ZodError, z } from 'zod';

export const ElectionObjectType = 'Election';
export const RegistrationRequestObjectType = 'RegistrationRequest';
export const RegistrationObjectType = 'Registration';
export const CastBallotObjectType = 'CastBallot';
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

export type JournalEntryAction = 'create' | 'delete' | string;

export const DateTimeSchema = z
  .string()
  .transform(DateTime.fromISO) as unknown as z.ZodSchema<DateTime>;

export class JournalEntry {
  constructor(
    private readonly id: Uuid,
    private readonly objectId: Uuid,
    private readonly jurisdictionCode: JurisdictionCode,
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

  getJurisdictionCode(): JurisdictionCode {
    return this.jurisdictionCode;
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
      jurisdictionCode: this.jurisdictionCode,
      objectType: this.objectType,
      action: this.action,
      createdAt: this.createdAt.toISO(),
    };
  }
}

export const JournalEntryStructSchema: z.ZodSchema<{
  id: Uuid;
  objectId: Uuid;
  jurisdictionCode: JurisdictionCode;
  objectType: string;
  action: string;
  createdAt: DateTime;
}> = z.object({
  id: UuidSchema,
  objectId: UuidSchema,
  jurisdictionCode: JurisdictionCodeSchema,
  objectType: z.string(),
  action: z.string(),
  createdAt: DateTimeSchema,
});

export const JournalEntrySchema: z.ZodSchema<JournalEntry> =
  JournalEntryStructSchema.transform(
    (o) =>
      new JournalEntry(
        o.id,
        o.objectId,
        o.jurisdictionCode,
        o.objectType,
        o.action,
        o.createdAt
      )
  ) as unknown as z.ZodSchema<JournalEntry>;

export class Election {
  constructor(
    private readonly jurisdictionCode: JurisdictionCode,
    private readonly electionDefinition: ElectionDefinition
  ) {}

  getJurisdictionCode(): JurisdictionCode {
    return this.jurisdictionCode;
  }

  getElectionDefinition(): ElectionDefinition {
    return this.electionDefinition;
  }

  toJSON(): unknown {
    return {
      jurisdictionCode: this.jurisdictionCode,
      electionDefinition: this.electionDefinition,
    };
  }
}

const ElectionStructSchema = z.object({
  jurisdictionCode: JurisdictionCodeSchema,
  electionDefinition: z
    .string()
    .transform((s) => Buffer.from(s, 'base64').toString('utf-8'))
    .transform((s) => safeParseElectionDefinition(s).unsafeUnwrap()),
});

export const ElectionSchema: z.ZodSchema<Election> =
  ElectionStructSchema.transform(
    (o) => new Election(o.jurisdictionCode, o.electionDefinition)
  ) as unknown as z.ZodSchema<Election>;

export class RegistrationRequest {
  constructor(
    private readonly commonAccessCardId: string,
    private readonly jurisdictionCode: JurisdictionCode,
    private readonly givenName: string,
    private readonly familyName: string,
    private readonly createdAt: DateTime
  ) {}

  getCommonAccessCardId(): string {
    return this.commonAccessCardId;
  }

  getJurisdictionCode(): JurisdictionCode {
    return this.jurisdictionCode;
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
      jurisdictionCode: this.jurisdictionCode,
      givenName: this.givenName,
      familyName: this.familyName,
      createdAt: this.createdAt.toISO(),
    };
  }
}

const RegistrationRequestStructSchema = z.object({
  commonAccessCardId: z.string(),
  jurisdictionCode: JurisdictionCodeSchema,
  givenName: z.string(),
  familyName: z.string(),
  createdAt: DateTimeSchema,
});

export const RegistrationRequestSchema: z.ZodSchema<RegistrationRequest> =
  RegistrationRequestStructSchema.transform(
    (o) =>
      new RegistrationRequest(
        o.commonAccessCardId,
        o.jurisdictionCode,
        o.givenName,
        o.familyName,
        o.createdAt
      )
  ) as unknown as z.ZodSchema<RegistrationRequest>;

export class Registration {
  constructor(
    private readonly commonAccessCardId: string,
    private readonly jurisdictionCode: JurisdictionCode,
    private readonly registrationRequestObjectId: Uuid,
    private readonly electionObjectId: Uuid,
    private readonly ballotStyleId: BallotStyleId,
    private readonly precinctId: PrecinctId
  ) {}

  getCommonAccessCardId(): string {
    return this.commonAccessCardId;
  }

  getJurisdictionCode(): JurisdictionCode {
    return this.jurisdictionCode;
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

const RegistrationStructSchema = z.object({
  commonAccessCardId: z.string(),
  jurisdictionCode: JurisdictionCodeSchema,
  registrationRequestObjectId: UuidSchema,
  electionObjectId: UuidSchema,
  ballotStyleId: BallotStyleIdSchema,
  precinctId: PrecinctIdSchema,
});

export const RegistrationSchema: z.ZodSchema<Registration> =
  RegistrationStructSchema.transform(
    (o) =>
      new Registration(
        o.commonAccessCardId,
        o.jurisdictionCode,
        o.registrationRequestObjectId,
        o.electionObjectId,
        o.ballotStyleId,
        o.precinctId
      )
  ) as unknown as z.ZodSchema<Registration>;

export class CastBallot {
  constructor(
    private readonly commonAccessCardId: string,
    private readonly jurisdictionCode: JurisdictionCode,
    private readonly registrationRequestObjectId: Uuid,
    private readonly registrationObjectId: Uuid,
    private readonly electionObjectId: Uuid,
    private readonly cvr: CVR.CVR
  ) {}

  getCommonAccessCardId(): string {
    return this.commonAccessCardId;
  }

  getJurisdictionCode(): JurisdictionCode {
    return this.jurisdictionCode;
  }

  getRegistrationRequestObjectId(): Uuid {
    return this.registrationRequestObjectId;
  }

  getRegistrationObjectId(): Uuid {
    return this.registrationObjectId;
  }

  getElectionObjectId(): Uuid {
    return this.electionObjectId;
  }

  getCVR(): CVR.CVR {
    return this.cvr;
  }
}

const CastBallotStructSchema = z.object({
  commonAccessCardId: z.string(),
  jurisdictionCode: JurisdictionCodeSchema,
  registrationRequestObjectId: UuidSchema,
  registrationObjectId: UuidSchema,
  electionObjectId: UuidSchema,
  cvr: CVR.CVRSchema,
});

export const CastBallotSchema: z.ZodSchema<CastBallot> =
  CastBallotStructSchema.transform(
    (o) =>
      new CastBallot(
        o.commonAccessCardId,
        o.jurisdictionCode,
        o.registrationRequestObjectId,
        o.registrationObjectId,
        o.electionObjectId,
        o.cvr
      )
  ) as unknown as z.ZodSchema<CastBallot>;

export type PayloadInner =
  | Election
  | Registration
  | RegistrationRequest
  | CastBallot;

export class Payload<T extends PayloadInner = PayloadInner> {
  constructor(
    private readonly objectType: string,
    private readonly data: T
  ) {}

  getObjectType(): string {
    return this.objectType;
  }

  getData(): T {
    return this.data;
  }

  toJSON(): unknown {
    // matches the format expected by the server and by `PayloadSchema`
    return {
      objectType: this.objectType,
      ...this.data,
    };
  }

  toBuffer(): Buffer {
    return Buffer.from(JSON.stringify(this));
  }

  static Election(data: Election): Payload<Election> {
    return new Payload(ElectionObjectType, data);
  }

  static Registration(data: Registration): Payload<Registration> {
    return new Payload(RegistrationObjectType, data);
  }

  static RegistrationRequest(
    data: RegistrationRequest
  ): Payload<RegistrationRequest> {
    return new Payload(RegistrationRequestObjectType, data);
  }

  static CastBallot(data: CastBallot): Payload<CastBallot> {
    return new Payload(CastBallotObjectType, data);
  }
}

export const PayloadSchema: z.ZodSchema<Payload> = z
  .discriminatedUnion('objectType', [
    z
      .object({ objectType: z.literal(ElectionObjectType) })
      .merge(ElectionStructSchema),
    z
      .object({ objectType: z.literal(RegistrationObjectType) })
      .merge(RegistrationStructSchema),
    z
      .object({ objectType: z.literal(RegistrationRequestObjectType) })
      .merge(RegistrationRequestStructSchema),
    z
      .object({ objectType: z.literal(CastBallotObjectType) })
      .merge(CastBallotStructSchema),
  ])
  .transform((o) => {
    switch (o.objectType) {
      case ElectionObjectType: {
        return Payload.Election(
          new Election(o.jurisdictionCode, o.electionDefinition)
        );
      }

      case RegistrationObjectType: {
        return Payload.Registration(
          new Registration(
            o.commonAccessCardId,
            o.jurisdictionCode,
            o.registrationRequestObjectId,
            o.electionObjectId,
            o.ballotStyleId,
            o.precinctId
          )
        );
      }

      case RegistrationRequestObjectType: {
        return Payload.RegistrationRequest(
          new RegistrationRequest(
            o.commonAccessCardId,
            o.jurisdictionCode,
            o.givenName,
            o.familyName,
            o.createdAt
          )
        );
      }

      case CastBallotObjectType: {
        return Payload.CastBallot(
          new CastBallot(
            o.commonAccessCardId,
            o.jurisdictionCode,
            o.registrationRequestObjectId,
            o.registrationObjectId,
            o.electionObjectId,
            o.cvr
          )
        );
      }

      default:
        throwIllegalValue(o);
    }
  }) as unknown as z.ZodSchema<Payload>;

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

  async getJurisdictionCode(): Promise<
    Result<JurisdictionCode, ZodError | SyntaxError>
  > {
    const parsePayloadResult = this.getPayload();
    if (parsePayloadResult.isOk()) {
      return ok(parsePayloadResult.ok().getData().getJurisdictionCode());
    }

    const fields = await certs.getCertSubjectFields(this.certificates);
    return safeParse(
      JurisdictionCodeSchema,
      fields.get(certs.VX_CUSTOM_CERT_FIELD.JURISDICTION)
    );
  }

  /**
   * Verify the signature on the payload against the embedded certificates.
   *
   * @returns `ok(true)` if the signature is valid, `ok(false)` if the signature
   * is invalid, or `err(Error)` if there was an error verifying the signature.
   */
  async verify(): Promise<Result<boolean, Error>> {
    try {
      const publicKey = await cryptography.extractPublicKeyFromCert(
        this.certificates
      );
      await cryptography.verifySignature({
        message: this.payload,
        messageSignature: this.signature,
        publicKey,
      });
      return ok(true);
    } catch (e) {
      return wrapException(e);
    }
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
    payload: z.string(),
    certificates: z.string(),
    signature: z.string(),
  })
  .transform(
    (o) =>
      new SignedObject(
        o.id,
        Buffer.from(o.payload, 'base64'),
        Buffer.from(o.certificates, 'base64'),
        Buffer.from(o.signature, 'base64')
      )
  ) as unknown as z.ZodSchema<SignedObject>;
