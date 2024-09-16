// eslint-disable-next-line max-classes-per-file
import { certs, cryptography } from '@votingworks/auth';
import {
  Optional,
  Result,
  err,
  ok,
  throwIllegalValue,
  wrapException,
} from '@votingworks/basics';
import {
  BallotStyleId,
  BallotStyleIdSchema,
  ElectionDefinition,
  NewType,
  PrecinctId,
  PrecinctIdSchema,
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
export type PayloadObjectType =
  | typeof ElectionObjectType
  | typeof RegistrationRequestObjectType
  | typeof RegistrationObjectType
  | typeof CastBallotObjectType;
export type Uuid = NewType<string, 'Uuid'>;

function propertyTransform<
  T extends object,
  K extends keyof T & string,
  S extends
    | z.ZodEffects<z.ZodTypeAny, unknown, unknown>
    | z.ZodOptional<z.ZodEffects<z.ZodTypeAny, unknown, unknown>>,
>(
  schema: S,
  key: K
): (
  arg: T,
  ctx: z.RefinementCtx
) => { [P in keyof T]: P extends K ? z.output<S> : T[P] } {
  return (o, ctx) => {
    const parsed = schema.safeParse(o[key]);

    if (parsed.error) {
      ctx.addIssue({
        code: 'custom',
        message: 'Invalid jurisdiction code',
        path: [key],
      });

      return z.NEVER;
    }

    // eslint-disable-next-line vx/gts-spread-like-types
    return { ...o, [key]: parsed.data } as unknown as {
      [P in keyof T]: P extends typeof key ? z.output<S> : T[P];
    };
  };
}

export function Uuid(): Uuid {
  return v4() as Uuid;
}

export const UuidSchema = z.string().transform((s, ctx) => {
  if (!validate(s)) {
    ctx.addIssue({
      code: 'custom',
      message: 'Invalid UUID',
    });

    return z.NEVER;
  }

  return s as Uuid;
});

export type JurisdictionCode = NewType<string, 'JurisdictionCode'>;

export const JurisdictionCodeSchema = z.string().transform((s, ctx) => {
  if (!/^[a-z]{2}\.[-_a-z0-9]+$/i.test(s)) {
    ctx.addIssue({
      code: 'custom',
      message: 'Invalid jurisdiction code',
    });

    return z.NEVER;
  }

  return s as JurisdictionCode;
});

export type JournalEntryAction = 'create' | 'delete' | string;

export const DateTimeSchema = z.string().transform((s) => DateTime.fromISO(s));

export class JournalEntry {
  constructor(
    private readonly id: Uuid,
    private readonly objectId: Uuid,
    // eslint-disable-next-line vx/gts-use-optionals
    private readonly electionId: Optional<Uuid>,
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

  getElectionId(): Optional<Uuid> {
    return this.electionId;
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

  toJSON(): JournalEntryStruct {
    return {
      id: this.id.toString(),
      objectId: this.objectId.toString(),
      electionId: this.electionId?.toString(),
      jurisdictionCode: this.jurisdictionCode,
      objectType: this.objectType,
      action: this.action,
      createdAt: this.createdAt.toISO(),
    };
  }
}

export interface JournalEntryStruct {
  id: string;
  objectId: string;
  electionId?: string;
  jurisdictionCode: string;
  objectType: string;
  action: string;
  createdAt: string;
}

export const JournalEntryStructSchema: z.ZodSchema<JournalEntryStruct> =
  z.object({
    id: z.string(),
    objectId: z.string(),
    electionId: z.string().optional(),
    jurisdictionCode: z.string(),
    objectType: z.string(),
    action: z.string(),
    createdAt: z.string(),
  });

export const JournalEntrySchema = JournalEntryStructSchema.transform(
  propertyTransform(UuidSchema, 'id')
)
  .transform(propertyTransform(UuidSchema, 'objectId'))
  .transform(propertyTransform(UuidSchema.optional(), 'electionId'))
  .transform(propertyTransform(JurisdictionCodeSchema, 'jurisdictionCode'))
  .transform(propertyTransform(DateTimeSchema, 'createdAt'))
  .transform(
    (o) =>
      new JournalEntry(
        o.id,
        o.objectId,
        o.electionId,
        o.jurisdictionCode,
        o.objectType,
        o.action,
        o.createdAt
      )
  );

export class Election {
  constructor(
    private readonly jurisdictionCode: JurisdictionCode,
    private readonly electionDefinition: ElectionDefinition,
    private readonly mailingAddress: string,
    private readonly electionguardElectionMetadataBlob: Buffer
  ) {}

  getJurisdictionCode(): JurisdictionCode {
    return this.jurisdictionCode;
  }

  getElectionDefinition(): ElectionDefinition {
    return this.electionDefinition;
  }

  getMailingAddress(): string {
    return this.mailingAddress;
  }

  getElectionguardElectionMetadataBlob(): Buffer {
    return this.electionguardElectionMetadataBlob;
  }

  toJSON(): ElectionStruct {
    return {
      jurisdictionCode: this.jurisdictionCode,
      electionDefinition: Buffer.from(
        this.electionDefinition.electionData
      ).toString('base64'),
      mailingAddress: this.mailingAddress,
      electionguardElectionMetadataBlob:
        this.electionguardElectionMetadataBlob.toString('base64'),
    };
  }
}

interface ElectionStruct {
  jurisdictionCode: string;
  electionDefinition: string;
  mailingAddress: string;
  electionguardElectionMetadataBlob: string;
}

const ElectionStructSchema = z.object({
  jurisdictionCode: z.string(),
  electionDefinition: z.string(),
  mailingAddress: z.string(),
  electionguardElectionMetadataBlob: z.string(),
}) satisfies z.ZodSchema<ElectionStruct>;

export const ElectionSchema = ElectionStructSchema.transform(
  (o) =>
    new Election(
      JurisdictionCodeSchema.parse(o.jurisdictionCode),
      safeParseElectionDefinition(
        Buffer.from(o.electionDefinition, 'base64').toString('utf-8')
      ).unsafeUnwrap(),
      o.mailingAddress,
      Buffer.from(o.electionguardElectionMetadataBlob, 'base64')
    )
);

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

  toJSON(): RegistrationRequestStruct {
    return {
      commonAccessCardId: this.commonAccessCardId,
      jurisdictionCode: this.jurisdictionCode,
      givenName: this.givenName,
      familyName: this.familyName,
      createdAt: this.createdAt.toISO(),
    };
  }
}

interface RegistrationRequestStruct {
  commonAccessCardId: string;
  jurisdictionCode: string;
  givenName: string;
  familyName: string;
  createdAt: string;
}

const RegistrationRequestStructSchema = z.object({
  commonAccessCardId: z.string(),
  jurisdictionCode: z.string(),
  givenName: z.string(),
  familyName: z.string(),
  createdAt: z.string(),
}) satisfies z.ZodSchema<RegistrationRequestStruct>;

export const RegistrationRequestSchema =
  RegistrationRequestStructSchema.transform(
    (o) =>
      new RegistrationRequest(
        o.commonAccessCardId,
        JurisdictionCodeSchema.parse(o.jurisdictionCode),
        o.givenName,
        o.familyName,
        DateTime.fromISO(o.createdAt)
      )
  );

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

  toJSON(): RegistrationStruct {
    return {
      commonAccessCardId: this.commonAccessCardId,
      jurisdictionCode: this.jurisdictionCode,
      registrationRequestObjectId: this.registrationRequestObjectId.toString(),
      electionObjectId: this.electionObjectId.toString(),
      ballotStyleId: this.ballotStyleId,
      precinctId: this.precinctId,
    };
  }
}

export interface RegistrationStruct {
  commonAccessCardId: string;
  jurisdictionCode: string;
  registrationRequestObjectId: string;
  electionObjectId: string;
  ballotStyleId: string;
  precinctId: string;
}

const RegistrationStructSchema = z.object({
  commonAccessCardId: z.string(),
  jurisdictionCode: JurisdictionCodeSchema,
  registrationRequestObjectId: UuidSchema,
  electionObjectId: UuidSchema,
  ballotStyleId: BallotStyleIdSchema,
  precinctId: PrecinctIdSchema,
}) satisfies z.ZodSchema<RegistrationStruct>;

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
    private readonly electionguardEncryptedBallot: Buffer
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

  getElectionguardEncryptedBallot(): Buffer {
    return this.electionguardEncryptedBallot;
  }

  toJSON(): CastBallotStruct {
    return {
      commonAccessCardId: this.commonAccessCardId,
      jurisdictionCode: this.jurisdictionCode,
      registrationRequestObjectId: this.registrationRequestObjectId.toString(),
      registrationObjectId: this.registrationObjectId.toString(),
      electionObjectId: this.electionObjectId.toString(),
      electionguardEncryptedBallot:
        this.electionguardEncryptedBallot.toString('base64'),
    };
  }
}

export interface CastBallotStruct {
  commonAccessCardId: string;
  jurisdictionCode: string;
  registrationRequestObjectId: string;
  registrationObjectId: string;
  electionObjectId: string;
  electionguardEncryptedBallot: string;
}

const CastBallotStructSchema = z.object({
  commonAccessCardId: z.string(),
  jurisdictionCode: z.string(),
  registrationRequestObjectId: z.string(),
  registrationObjectId: z.string(),
  electionObjectId: z.string(),
  electionguardEncryptedBallot: z.string(),
}) satisfies z.ZodSchema<CastBallotStruct>;

const Base64BufferSchema = z.string().transform((s, ctx) => {
  try {
    return Buffer.from(s, 'base64');
  } catch (e) {
    ctx.addIssue({
      code: 'custom',
      message: 'Invalid base64 encoding',
    });

    return z.NEVER;
  }
});

export const CastBallotSchema = CastBallotStructSchema.transform(
  propertyTransform(JurisdictionCodeSchema, 'jurisdictionCode')
)
  .transform(propertyTransform(UuidSchema, 'registrationRequestObjectId'))
  .transform(propertyTransform(UuidSchema, 'registrationObjectId'))
  .transform(propertyTransform(UuidSchema, 'electionObjectId'))
  .transform(
    propertyTransform(Base64BufferSchema, 'electionguardEncryptedBallot')
  )
  .transform((o) => {
    return new CastBallot(
      o.commonAccessCardId,
      o.jurisdictionCode,
      o.registrationRequestObjectId,
      o.registrationObjectId,
      o.electionObjectId,
      o.electionguardEncryptedBallot
    );
  });

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
      ...this.data.toJSON(),
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
        return Payload.Election(ElectionSchema.parse(o));
      }

      case RegistrationObjectType: {
        return Payload.Registration(RegistrationSchema.parse(o));
      }

      case RegistrationRequestObjectType: {
        return Payload.RegistrationRequest(RegistrationRequestSchema.parse(o));
      }

      case CastBallotObjectType: {
        return Payload.CastBallot(CastBallotSchema.parse(o));
      }

      default:
        throwIllegalValue(o);
    }
  }) as unknown as z.ZodSchema<Payload>;

export class SignedObject {
  constructor(
    private readonly id: Uuid,
    // eslint-disable-next-line vx/gts-use-optionals
    private readonly electionId: Optional<Uuid>,
    private readonly payload: Buffer,
    private readonly certificate: Buffer,
    private readonly signature: Buffer
  ) {}

  getId(): Uuid {
    return this.id;
  }

  getElectionId(): Optional<Uuid> {
    return this.electionId;
  }

  getPayloadRaw(): Buffer {
    return this.payload;
  }

  getPayload(): Result<Payload, ZodError | SyntaxError> {
    return safeParseJson(this.payload.toString(), PayloadSchema);
  }

  getPayloadAsObjectType(
    objectType: typeof ElectionObjectType
  ): Result<Payload<Election>, ZodError | SyntaxError>;
  getPayloadAsObjectType(
    objectType: typeof RegistrationRequestObjectType
  ): Result<Payload<RegistrationRequest>, ZodError | SyntaxError>;
  getPayloadAsObjectType(
    objectType: typeof RegistrationObjectType
  ): Result<Payload<Registration>, ZodError | SyntaxError>;
  getPayloadAsObjectType(
    objectType: typeof CastBallotObjectType
  ): Result<Payload<CastBallot>, ZodError | SyntaxError>;
  // eslint-disable-next-line vx/gts-no-return-type-only-generics
  getPayloadAsObjectType<T extends PayloadInner>(
    objectType: PayloadObjectType
  ): Result<Payload<T>, ZodError | SyntaxError> {
    const parsePayloadResult = this.getPayload();
    if (parsePayloadResult.isErr()) {
      return parsePayloadResult;
    }

    const payload = parsePayloadResult.ok();
    if (payload.getObjectType() === objectType) {
      return ok(payload as Payload<T>);
    }

    return err(
      new ZodError([
        {
          code: 'custom',
          message: `Expected payload object type ${objectType}, got ${payload.getObjectType()}`,
          path: ['objectType'],
        },
      ])
    );
  }

  getCertificate(): Buffer {
    return this.certificate;
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

    const fields = await certs.getCertSubjectFields(this.certificate);
    return ok(
      JurisdictionCodeSchema.parse(
        fields.get(certs.VX_CUSTOM_CERT_FIELD.JURISDICTION)
      )
    );
  }

  /**
   * Verify the signature on the payload against the embedded certificate.
   *
   * @returns `ok(true)` if the signature is valid, `ok(false)` if the signature
   * is invalid, or `err(Error)` if there was an error verifying the signature.
   */
  async verify(): Promise<Result<boolean, Error>> {
    try {
      const publicKey = await cryptography.extractPublicKeyFromCert(
        this.certificate
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
      electionId: this.electionId?.toString(),
      payload: this.payload.toString('base64'),
      certificate: this.certificate.toString('base64'),
      signature: this.signature.toString('base64'),
    };
  }
}

export const SignedObjectSchema: z.ZodSchema<SignedObject> = z
  .object({
    id: UuidSchema,
    electionId: UuidSchema.optional(),
    payload: z.string(),
    certificate: z.string(),
    signature: z.string(),
  })
  .transform(
    (o) =>
      new SignedObject(
        o.id,
        o.electionId,
        Buffer.from(o.payload, 'base64'),
        Buffer.from(o.certificate, 'base64'),
        Buffer.from(o.signature, 'base64')
      )
  ) as unknown as z.ZodSchema<SignedObject>;

export class CreateSessionRequestPayload {
  constructor(private readonly timestamp: DateTime) {}

  getTimestamp(): DateTime {
    return this.timestamp;
  }

  toJSON(): CreateSessionRequestPayloadStruct {
    return {
      timestamp: this.timestamp.toISO(),
    };
  }
}

export interface CreateSessionRequestPayloadStruct {
  timestamp: string;
}

export const CreateSessionRequestPayloadSchema = z
  .object({
    timestamp: z.string(),
  })
  .transform(
    (o) => new CreateSessionRequestPayload(DateTime.fromISO(o.timestamp))
  ) as unknown as z.ZodSchema<CreateSessionRequestPayload>;

export class CreateSessionRequest {
  constructor(
    private readonly certificate: Buffer,
    private readonly payload: string,
    private readonly signature: Buffer
  ) {}

  /**
   * A PEM-encoded X.509 certificate. Contains the client TPM's public key
   * certificate as signed by the CA given in the cacvote-server configuration.
   */
  getCertificate(): Buffer {
    return this.certificate;
  }

  /**
   * The payload of the request. Must be JSON decodable as
   * {@link CreateSessionRequestPayload}.
   */
  getPayload(): Result<CreateSessionRequestPayload, ZodError | SyntaxError> {
    return safeParseJson(this.payload, CreateSessionRequestPayloadSchema);
  }

  /**
   * The signature of the payload as signed by the client's TPM.
   */
  getSignature(): Buffer {
    return this.signature;
  }

  toJSON(): CreateSessionRequestStruct {
    return {
      certificate: this.certificate.toString('base64'),
      payload: this.payload,
      signature: this.signature.toString('base64'),
    };
  }
}

export interface CreateSessionRequestStruct {
  certificate: string;
  payload: string;
  signature: string;
}

export const CreateSessionRequestStructSchema = z.object({
  certificate: z.string(),
  payload: z.string(),
  signature: z.string(),
});

export const CreateSessionRequestSchema =
  CreateSessionRequestStructSchema.transform(
    propertyTransform(Base64BufferSchema, 'certificate')
  )
    .transform(propertyTransform(Base64BufferSchema, 'signature'))
    .transform(
      (o) => new CreateSessionRequest(o.certificate, o.payload, o.signature)
    ) as unknown as z.ZodSchema<CreateSessionRequest>;

export class CreateSessionResponse {
  constructor(private readonly bearerToken: string) {}

  getBearerToken(): string {
    return this.bearerToken;
  }

  toJSON(): CreateSessionResponseStruct {
    return {
      bearerToken: this.bearerToken,
    };
  }
}

export interface CreateSessionResponseStruct {
  bearerToken: string;
}

export const CreateSessionResponseStructSchema = z.object({
  bearerToken: z.string(),
});

export const CreateSessionResponseSchema =
  CreateSessionResponseStructSchema.transform(
    (o) => new CreateSessionResponse(o.bearerToken)
  ) as unknown as z.ZodSchema<CreateSessionResponse>;
