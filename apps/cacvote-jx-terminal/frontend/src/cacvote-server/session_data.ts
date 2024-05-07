/* eslint-disable max-classes-per-file */
import { Optional } from '@votingworks/basics';
import {
  BallotStyleId,
  BallotStyleIdSchema,
  ElectionDefinition,
  PrecinctId,
  PrecinctIdSchema,
  safeParseElectionDefinition,
  unsafeParse,
} from '@votingworks/types';
import { Buffer } from 'buffer';
import { DateTime } from 'luxon';
import { z } from 'zod';
import {
  Iso8601DateSchema,
  JurisdictionCode,
  JurisdictionCodeSchema,
  Uuid,
  UuidSchema,
} from './types';

export interface VerificationStatusSuccess {
  type: 'success';
  commonAccessCardId: string;
  displayName: string;
}

export const VerificationStatusSuccessSchema: z.ZodSchema<VerificationStatusSuccess> =
  z.object({
    type: z.literal('success'),
    commonAccessCardId: z.string(),
    displayName: z.string(),
  });

export interface VerificationStatusFailure {
  type: 'failure';
}

export const VerificationStatusFailureSchema: z.ZodSchema<VerificationStatusFailure> =
  z.object({
    type: z.literal('failure'),
  });

export interface VerificationStatusError {
  type: 'error';
  message: string;
}

export const VerificationStatusErrorSchema: z.ZodSchema<VerificationStatusError> =
  z.object({
    type: z.literal('error'),
    message: z.string(),
  });

export interface VerificationStatusUnknown {
  type: 'unknown';
}

export const VerificationStatusUnknownSchema: z.ZodSchema<VerificationStatusUnknown> =
  z.object({
    type: z.literal('unknown'),
  });

export type VerificationStatus =
  | VerificationStatusSuccess
  | VerificationStatusFailure
  | VerificationStatusError
  | VerificationStatusUnknown;

export const VerificationStatusSchema: z.ZodSchema<VerificationStatus> =
  z.union([
    VerificationStatusSuccessSchema,
    VerificationStatusFailureSchema,
    VerificationStatusErrorSchema,
    VerificationStatusUnknownSchema,
  ]);

export interface EncryptedElectionTallyStruct {
  jurisdictionCode: string;
  electionObjectId: string;
  electionguardEncryptedTally: string;
}

export const EncryptedElectionTallyStructSchema: z.ZodSchema<EncryptedElectionTallyStruct> =
  z.object({
    jurisdictionCode: z.string(),
    electionObjectId: z.string(),
    electionguardEncryptedTally: z.string(),
  });

export class EncryptedElectionTally {
  constructor(
    private readonly jurisdictionCode: JurisdictionCode,
    private readonly electionObjectId: Uuid,
    private readonly electionguardEncryptedTally: Buffer
  ) {}

  getJurisdictionCode(): JurisdictionCode {
    return this.jurisdictionCode;
  }

  getElectionObjectId(): Uuid {
    return this.electionObjectId;
  }

  getElectionguardEncryptedTally(): Buffer {
    return this.electionguardEncryptedTally;
  }

  toJSON(): EncryptedElectionTallyStruct {
    return {
      jurisdictionCode: this.jurisdictionCode,
      electionObjectId: this.electionObjectId,
      electionguardEncryptedTally:
        this.electionguardEncryptedTally.toString('base64'),
    };
  }
}

export const EncryptedElectionTallySchema: z.ZodSchema<EncryptedElectionTally> =
  EncryptedElectionTallyStructSchema.transform(
    (struct) =>
      new EncryptedElectionTally(
        unsafeParse(JurisdictionCodeSchema, struct.jurisdictionCode),
        unsafeParse(UuidSchema, struct.electionObjectId),
        Buffer.from(struct.electionguardEncryptedTally, 'base64')
      )
  ) as unknown as z.ZodSchema<EncryptedElectionTally>;

export interface EncryptedElectionTallyPresenterStruct {
  encryptedElectionTally: EncryptedElectionTallyStruct;
  createdAt: string;
  syncedAt?: string;
}

export const EncryptedElectionTallyPresenterStructSchema: z.ZodSchema<EncryptedElectionTallyPresenterStruct> =
  z.object({
    encryptedElectionTally: EncryptedElectionTallyStructSchema,
    createdAt: z.string(),
    syncedAt: z.string().optional(),
  });

export class EncryptedElectionTallyPresenter {
  constructor(
    private readonly encryptedElectionTally: EncryptedElectionTally,
    private readonly createdAt: DateTime,
    private readonly syncedAt?: DateTime
  ) {}

  getEncryptedElectionTally(): EncryptedElectionTally {
    return this.encryptedElectionTally;
  }

  getCreatedAt(): DateTime {
    return this.createdAt;
  }

  getSyncedAt(): DateTime | undefined {
    return this.syncedAt;
  }

  toJSON(): EncryptedElectionTallyPresenterStruct {
    return {
      encryptedElectionTally: this.encryptedElectionTally.toJSON(),
      createdAt: this.createdAt.toISO(),
      syncedAt: this.syncedAt?.toISO(),
    };
  }
}

export const EncryptedElectionTallyPresenterSchema: z.ZodSchema<EncryptedElectionTallyPresenter> =
  EncryptedElectionTallyPresenterStructSchema.transform(
    (struct) =>
      new EncryptedElectionTallyPresenter(
        unsafeParse(
          EncryptedElectionTallySchema,
          struct.encryptedElectionTally
        ),
        DateTime.fromISO(struct.createdAt),
        struct.syncedAt ? DateTime.fromISO(struct.syncedAt) : undefined
      )
  ) as unknown as z.ZodSchema<EncryptedElectionTallyPresenter>;

export interface DecryptedElectionTallyStruct {
  jurisdictionCode: string;
  electionObjectId: string;
  electionguardDecryptedTally: string;
}

export const DecryptedElectionTallyStructSchema: z.ZodSchema<DecryptedElectionTallyStruct> =
  z.object({
    jurisdictionCode: z.string(),
    electionObjectId: z.string(),
    electionguardDecryptedTally: z.string(),
  });

export class DecryptedElectionTally {
  constructor(
    private readonly jurisdictionCode: JurisdictionCode,
    private readonly electionObjectId: Uuid,
    private readonly electionguardDecryptedTally: Buffer
  ) {}

  getJurisdictionCode(): JurisdictionCode {
    return this.jurisdictionCode;
  }

  getElectionObjectId(): Uuid {
    return this.electionObjectId;
  }

  getElectionguardDecryptedTally(): Buffer {
    return this.electionguardDecryptedTally;
  }

  toJSON(): DecryptedElectionTallyStruct {
    return {
      jurisdictionCode: this.jurisdictionCode,
      electionObjectId: this.electionObjectId,
      electionguardDecryptedTally:
        this.electionguardDecryptedTally.toString('base64'),
    };
  }
}

export const DecryptedElectionTallySchema: z.ZodSchema<DecryptedElectionTally> =
  DecryptedElectionTallyStructSchema.transform(
    (struct) =>
      new DecryptedElectionTally(
        unsafeParse(JurisdictionCodeSchema, struct.jurisdictionCode),
        unsafeParse(UuidSchema, struct.electionObjectId),
        Buffer.from(struct.electionguardDecryptedTally, 'base64')
      )
  ) as unknown as z.ZodSchema<DecryptedElectionTally>;

export interface DecryptedElectionTallyPresenterStruct {
  decryptedElectionTally: DecryptedElectionTallyStruct;
  createdAt: string;
  syncedAt?: string;
}

export const DecryptedElectionTallyPresenterStructSchema: z.ZodSchema<DecryptedElectionTallyPresenterStruct> =
  z.object({
    decryptedElectionTally: DecryptedElectionTallyStructSchema,
    createdAt: z.string(),
    syncedAt: z.string().optional(),
  });

export class DecryptedElectionTallyPresenter {
  constructor(
    private readonly decryptedElectionTally: DecryptedElectionTally,
    private readonly createdAt: DateTime,
    private readonly syncedAt?: DateTime
  ) {}

  getDecryptedElectionTally(): DecryptedElectionTally {
    return this.decryptedElectionTally;
  }

  getCreatedAt(): DateTime {
    return this.createdAt;
  }

  getSyncedAt(): DateTime | undefined {
    return this.syncedAt;
  }

  toJSON(): DecryptedElectionTallyPresenterStruct {
    return {
      decryptedElectionTally: this.decryptedElectionTally.toJSON(),
      createdAt: this.createdAt.toISO(),
      syncedAt: this.syncedAt?.toISO(),
    };
  }
}

export const DecryptedElectionTallyPresenterSchema: z.ZodSchema<DecryptedElectionTallyPresenter> =
  DecryptedElectionTallyPresenterStructSchema.transform(
    (struct) =>
      new DecryptedElectionTallyPresenter(
        unsafeParse(
          DecryptedElectionTallySchema,
          struct.decryptedElectionTally
        ),
        DateTime.fromISO(struct.createdAt),
        struct.syncedAt ? DateTime.fromISO(struct.syncedAt) : undefined
      )
  ) as unknown as z.ZodSchema<DecryptedElectionTallyPresenter>;

export interface ShuffledEncryptedCastBallotsPresenterStruct {
  jurisdictionCode: string;
  electionObjectId: string;
  electionguardShuffledBallots: string;
  createdAt: string;
  syncedAt?: string;
}

export const ShuffledEncryptedCastBallotsPresenterStructSchema: z.ZodSchema<ShuffledEncryptedCastBallotsPresenterStruct> =
  z.object({
    jurisdictionCode: z.string(),
    electionObjectId: z.string(),
    electionguardShuffledBallots: z.string(),
    createdAt: z.string(),
    syncedAt: z.string().optional(),
  });

export class ShuffledEncryptedCastBallotsPresenter {
  constructor(
    private readonly jurisdictionCode: JurisdictionCode,
    private readonly electionObjectId: Uuid,
    private readonly electionguardShuffledBallots: Buffer,
    private readonly createdAt: DateTime,
    private readonly syncedAt?: DateTime
  ) {}

  getJurisdictionCode(): JurisdictionCode {
    return this.jurisdictionCode;
  }

  getElectionObjectId(): Uuid {
    return this.electionObjectId;
  }

  getElectionguardShuffledBallots(): Buffer {
    return this.electionguardShuffledBallots;
  }

  getCreatedAt(): DateTime {
    return this.createdAt;
  }

  getSyncedAt(): Optional<DateTime> {
    return this.syncedAt;
  }

  toJSON(): ShuffledEncryptedCastBallotsPresenterStruct {
    return {
      jurisdictionCode: this.jurisdictionCode,
      electionObjectId: this.electionObjectId,
      electionguardShuffledBallots:
        this.electionguardShuffledBallots.toString('base64'),
      createdAt: this.createdAt.toISO(),
      syncedAt: this.syncedAt?.toISO(),
    };
  }
}

export const ShuffledEncryptedCastBallotsPresenterSchema: z.ZodSchema<ShuffledEncryptedCastBallotsPresenter> =
  ShuffledEncryptedCastBallotsPresenterStructSchema.transform(
    (struct) =>
      new ShuffledEncryptedCastBallotsPresenter(
        unsafeParse(JurisdictionCodeSchema, struct.jurisdictionCode),
        unsafeParse(UuidSchema, struct.electionObjectId),
        Buffer.from(struct.electionguardShuffledBallots, 'base64'),
        DateTime.fromISO(struct.createdAt),
        struct.syncedAt ? DateTime.fromISO(struct.syncedAt) : undefined
      )
  ) as unknown as z.ZodSchema<ShuffledEncryptedCastBallotsPresenter>;

export interface ElectionInfoStruct {
  jurisdictionCode: string;
  electionDefinition: string;
  mailingAddress: string;
  electionguardElectionMetadataBlob: string;
}

export const ElectionInfoStructSchema: z.ZodSchema<ElectionInfoStruct> =
  z.object({
    jurisdictionCode: z.string(),
    electionDefinition: z.string(),
    mailingAddress: z.string(),
    electionguardElectionMetadataBlob: z.string(),
  });

export class ElectionInfo {
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

  toJSON(): ElectionInfoStruct {
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

export const ElectionInfoSchema: z.ZodSchema<ElectionInfo> =
  ElectionInfoStructSchema.transform(
    (struct) =>
      new ElectionInfo(
        unsafeParse(JurisdictionCodeSchema, struct.jurisdictionCode),
        safeParseElectionDefinition(
          Buffer.from(struct.electionDefinition, 'base64').toString('utf-8')
        ).unsafeUnwrap(),
        struct.mailingAddress,
        Buffer.from(struct.electionguardElectionMetadataBlob, 'base64')
      )
  ) as unknown as z.ZodSchema<ElectionInfo>;

export interface ElectionPresenterStruct {
  id: string;
  election: ElectionInfoStruct;
  encryptedTally?: EncryptedElectionTallyPresenterStruct;
  decryptedTally?: DecryptedElectionTallyPresenterStruct;
  shuffledEncryptedCastBallots?: ShuffledEncryptedCastBallotsPresenterStruct;
}

export const ElectionPresenterStructSchema: z.ZodSchema<ElectionPresenterStruct> =
  z.object({
    id: z.string(),
    election: ElectionInfoStructSchema,
    encryptedTally: EncryptedElectionTallyPresenterStructSchema.optional(),
    decryptedTally: DecryptedElectionTallyPresenterStructSchema.optional(),
    shuffledEncryptedCastBallots:
      ShuffledEncryptedCastBallotsPresenterStructSchema.optional(),
  });

export class ElectionPresenter {
  constructor(
    private readonly id: Uuid,
    private readonly election: ElectionInfo,
    private readonly encryptedTally?: EncryptedElectionTallyPresenter,
    private readonly decryptedTally?: DecryptedElectionTallyPresenter,
    private readonly shuffledEncryptedCastBallots?: ShuffledEncryptedCastBallotsPresenter
  ) {}

  getId(): Uuid {
    return this.id;
  }

  getElection(): ElectionInfo {
    return this.election;
  }

  getEncryptedTally(): Optional<EncryptedElectionTallyPresenter> {
    return this.encryptedTally;
  }

  getDecryptedTally(): Optional<DecryptedElectionTallyPresenter> {
    return this.decryptedTally;
  }

  getShuffledEncryptedCastBallots(): Optional<ShuffledEncryptedCastBallotsPresenter> {
    return this.shuffledEncryptedCastBallots;
  }

  toJSON(): ElectionPresenterStruct {
    return {
      id: this.id,
      election: this.election.toJSON(),
      encryptedTally: this.encryptedTally?.toJSON(),
      decryptedTally: this.decryptedTally?.toJSON(),
      shuffledEncryptedCastBallots: this.shuffledEncryptedCastBallots?.toJSON(),
    };
  }
}

export const ElectionPresenterSchema: z.ZodSchema<ElectionPresenter> =
  ElectionPresenterStructSchema.transform(
    (struct) =>
      new ElectionPresenter(
        unsafeParse(UuidSchema, struct.id),
        unsafeParse(ElectionInfoSchema, struct.election),
        struct.encryptedTally &&
          unsafeParse(
            EncryptedElectionTallyPresenterSchema,
            struct.encryptedTally
          ),
        struct.decryptedTally &&
          unsafeParse(
            DecryptedElectionTallyPresenterSchema,
            struct.decryptedTally
          ),
        struct.shuffledEncryptedCastBallots &&
          unsafeParse(
            ShuffledEncryptedCastBallotsPresenterSchema,
            struct.shuffledEncryptedCastBallots
          )
      )
  ) as unknown as z.ZodSchema<ElectionPresenter>;

export interface RegistrationRequestStruct {
  commonAccessCardId: string;
  jurisdictionCode: string;
  givenName: string;
  familyName: string;
}

export const RegistrationRequestStructSchema: z.ZodSchema<RegistrationRequestStruct> =
  z.object({
    commonAccessCardId: z.string(),
    jurisdictionCode: z.string(),
    givenName: z.string(),
    familyName: z.string(),
  });

export class RegistrationRequest {
  constructor(
    private readonly commonAccessCardId: string,
    private readonly jurisdictionCode: JurisdictionCode,
    private readonly givenName: string,
    private readonly familyName: string
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

  toJSON(): RegistrationRequestStruct {
    return {
      commonAccessCardId: this.commonAccessCardId,
      jurisdictionCode: this.jurisdictionCode,
      givenName: this.givenName,
      familyName: this.familyName,
    };
  }
}

export const RegistrationRequestSchema: z.ZodSchema<RegistrationRequest> =
  RegistrationRequestStructSchema.transform(
    (struct) =>
      new RegistrationRequest(
        struct.commonAccessCardId,
        unsafeParse(JurisdictionCodeSchema, struct.jurisdictionCode),
        struct.givenName,
        struct.familyName
      )
  ) as unknown as z.ZodSchema<RegistrationRequest>;

export interface RegistrationRequestPresenter {
  id: Uuid;
  displayName: string;
  registrationRequest: RegistrationRequestStruct;
  createdAt: DateTime;
}

export const RegistrationRequestPresenterSchema: z.ZodSchema<RegistrationRequestPresenter> =
  z.object({
    id: UuidSchema,
    displayName: z.string(),
    registrationRequest: RegistrationRequestStructSchema,
    createdAt: Iso8601DateSchema,
  });

export interface RegistrationStruct {
  commonAccessCardId: string;
  jurisdictionCode: string;
  registrationRequestObjectId: string;
  electionObjectId: string;
  ballotStyleId: string;
  precinctId: string;
}

export const RegistrationStructSchema: z.ZodSchema<RegistrationStruct> =
  z.object({
    commonAccessCardId: z.string(),
    jurisdictionCode: JurisdictionCodeSchema,
    registrationRequestObjectId: z.string(),
    electionObjectId: z.string(),
    ballotStyleId: z.string(),
    precinctId: z.string(),
  });

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
      registrationRequestObjectId: this.registrationRequestObjectId,
      electionObjectId: this.electionObjectId,
      ballotStyleId: this.ballotStyleId,
      precinctId: this.precinctId,
    };
  }
}

export const RegistrationSchema: z.ZodSchema<Registration> =
  RegistrationStructSchema.transform(
    (struct) =>
      new Registration(
        struct.commonAccessCardId,
        unsafeParse(JurisdictionCodeSchema, struct.jurisdictionCode),
        unsafeParse(UuidSchema, struct.registrationRequestObjectId),
        unsafeParse(UuidSchema, struct.electionObjectId),
        unsafeParse(BallotStyleIdSchema, struct.ballotStyleId),
        unsafeParse(PrecinctIdSchema, struct.precinctId)
      )
  ) as unknown as z.ZodSchema<Registration>;

export interface RegistrationPresenterStruct {
  id: Uuid;
  displayName: string;
  electionTitle: string;
  electionHash: string;
  registration: RegistrationStruct;
  createdAt: string;
  isSynced: boolean;
}

export const RegistrationPresenterStructSchema: z.ZodSchema<RegistrationPresenterStruct> =
  z.object({
    id: UuidSchema,
    displayName: z.string(),
    electionTitle: z.string(),
    electionHash: z.string(),
    registration: RegistrationStructSchema,
    createdAt: z.string(),
    isSynced: z.boolean(),
  });

export class RegistrationPresenter {
  constructor(
    private readonly id: Uuid,
    private readonly displayName: string,
    private readonly electionTitle: string,
    private readonly electionHash: string,
    private readonly registration: Registration,
    private readonly createdAt: DateTime,
    private readonly isSynced: boolean
  ) {}

  getId(): Uuid {
    return this.id;
  }

  getDisplayName(): string {
    return this.displayName;
  }

  getElectionTitle(): string {
    return this.electionTitle;
  }

  getElectionHash(): string {
    return this.electionHash;
  }

  getRegistration(): Registration {
    return this.registration;
  }

  getCreatedAt(): DateTime {
    return this.createdAt;
  }

  getIsSynced(): boolean {
    return this.isSynced;
  }

  toJSON(): RegistrationPresenterStruct {
    return {
      id: this.id,
      displayName: this.displayName,
      electionTitle: this.electionTitle,
      electionHash: this.electionHash,
      registration: this.registration.toJSON(),
      createdAt: this.createdAt.toISO(),
      isSynced: this.isSynced,
    };
  }
}

export const RegistrationPresenterSchema: z.ZodSchema<RegistrationPresenter> =
  RegistrationPresenterStructSchema.transform(
    (struct) =>
      new RegistrationPresenter(
        struct.id,
        struct.displayName,
        struct.electionTitle,
        struct.electionHash,
        unsafeParse(RegistrationSchema, struct.registration),
        DateTime.fromISO(struct.createdAt),
        struct.isSynced
      )
  ) as unknown as z.ZodSchema<RegistrationPresenter>;

export interface CastBallot {
  commonAccessCardId: string;
  jurisdictionCode: JurisdictionCode;
  registrationRequestObjectId: Uuid;
  registrationObjectId: Uuid;
  electionObjectId: Uuid;
  electionguardEncryptedBallot: string;
}

export const CastBallotSchema: z.ZodSchema<CastBallot> = z.object({
  commonAccessCardId: z.string(),
  jurisdictionCode: JurisdictionCodeSchema,
  registrationRequestObjectId: UuidSchema,
  registrationObjectId: UuidSchema,
  electionObjectId: UuidSchema,
  electionguardEncryptedBallot: z.string(),
});

export interface CastBallotPresenterStruct {
  castBallot: CastBallot;
  registrationRequest: RegistrationRequestStruct;
  registration: RegistrationStruct;
  registrationId: string;
  verificationStatus: VerificationStatus;
  createdAt: string;
}

export const CastBallotPresenterStructSchema: z.ZodSchema<CastBallotPresenterStruct> =
  z.object({
    castBallot: CastBallotSchema,
    registrationRequest: RegistrationRequestStructSchema,
    registration: RegistrationStructSchema,
    registrationId: z.string(),
    verificationStatus: VerificationStatusSchema,
    createdAt: z.string(),
  });

export class CastBallotPresenter {
  constructor(
    private readonly castBallot: CastBallot,
    private readonly registrationRequest: RegistrationRequest,
    private readonly registration: Registration,
    private readonly registrationId: Uuid,
    private readonly verificationStatus: VerificationStatus,
    private readonly createdAt: DateTime
  ) {}

  getCastBallot(): CastBallot {
    return this.castBallot;
  }

  getRegistrationRequest(): RegistrationRequest {
    return this.registrationRequest;
  }

  getRegistration(): Registration {
    return this.registration;
  }

  getRegistrationId(): Uuid {
    return this.registrationId;
  }

  getVerificationStatus(): VerificationStatus {
    return this.verificationStatus;
  }

  getCreatedAt(): DateTime {
    return this.createdAt;
  }

  toJSON(): CastBallotPresenterStruct {
    return {
      castBallot: this.castBallot,
      registrationRequest: this.registrationRequest.toJSON(),
      registration: this.registration.toJSON(),
      registrationId: this.registrationId,
      verificationStatus: this.verificationStatus,
      createdAt: this.createdAt.toISO(),
    };
  }
}

export const CastBallotPresenterSchema: z.ZodSchema<CastBallotPresenter> =
  CastBallotPresenterStructSchema.transform(
    (struct) =>
      new CastBallotPresenter(
        struct.castBallot,
        unsafeParse(RegistrationRequestSchema, struct.registrationRequest),
        unsafeParse(RegistrationSchema, struct.registration),
        unsafeParse(UuidSchema, struct.registrationId),
        struct.verificationStatus,
        DateTime.fromISO(struct.createdAt)
      )
  ) as unknown as z.ZodSchema<CastBallotPresenter>;

export interface AuthenticatedSessionDataStruct {
  type: 'authenticated';
  jurisdictionCode: JurisdictionCode;
  elections: ElectionPresenterStruct[];
  pendingRegistrationRequests: RegistrationRequestPresenter[];
  registrations: RegistrationPresenterStruct[];
  castBallots: CastBallotPresenterStruct[];
}

export const AuthenticatedSessionDataStructSchema: z.ZodSchema<AuthenticatedSessionDataStruct> =
  z.object({
    type: z.literal('authenticated'),
    jurisdictionCode: JurisdictionCodeSchema,
    elections: z.array(ElectionPresenterStructSchema),
    pendingRegistrationRequests: z.array(RegistrationRequestPresenterSchema),
    registrations: z.array(RegistrationPresenterStructSchema),
    castBallots: z.array(CastBallotPresenterStructSchema),
  });

export class AuthenticatedSessionData {
  constructor(
    private readonly jurisdictionCode: JurisdictionCode,
    private readonly elections: ElectionPresenter[],
    private readonly pendingRegistrationRequests: RegistrationRequestPresenter[],
    private readonly registrations: RegistrationPresenter[],
    private readonly castBallots: CastBallotPresenter[]
  ) {}

  get type(): 'authenticated' {
    return 'authenticated';
  }

  getJurisdictionCode(): JurisdictionCode {
    return this.jurisdictionCode;
  }

  getElections(): ElectionPresenter[] {
    return this.elections;
  }

  getPendingRegistrationRequests(): RegistrationRequestPresenter[] {
    return this.pendingRegistrationRequests;
  }

  getRegistrations(): RegistrationPresenter[] {
    return this.registrations;
  }

  getCastBallots(): CastBallotPresenter[] {
    return this.castBallots;
  }

  toJSON(): AuthenticatedSessionDataStruct {
    return {
      type: 'authenticated',
      jurisdictionCode: this.jurisdictionCode,
      elections: this.elections.map((e) => e.toJSON()),
      pendingRegistrationRequests: this.pendingRegistrationRequests,
      registrations: this.registrations.map((r) => r.toJSON()),
      castBallots: this.castBallots.map((c) => c.toJSON()),
    };
  }
}

export const AuthenticatedSessionDataSchema: z.ZodSchema<AuthenticatedSessionData> =
  AuthenticatedSessionDataStructSchema.transform(
    (struct) =>
      new AuthenticatedSessionData(
        struct.jurisdictionCode,
        struct.elections.map((e) => unsafeParse(ElectionPresenterSchema, e)),
        struct.pendingRegistrationRequests,
        struct.registrations.map((r) =>
          unsafeParse(RegistrationPresenterSchema, r)
        ),
        struct.castBallots.map((c) => unsafeParse(CastBallotPresenterSchema, c))
      )
  ) as unknown as z.ZodSchema<AuthenticatedSessionData>;

export interface UnauthenticatedSessionDataStruct {
  type: 'unauthenticated';
  hasSmartcard: boolean;
}

export const UnauthenticatedSessionDataStructSchema: z.ZodSchema<UnauthenticatedSessionDataStruct> =
  z.object({
    type: z.literal('unauthenticated'),
    hasSmartcard: z.boolean(),
  });

export class UnauthenticatedSessionData {
  constructor(private readonly hasSmartcard: boolean) {}

  getHasSmartcard(): boolean {
    return this.hasSmartcard;
  }

  toJSON(): UnauthenticatedSessionDataStruct {
    return {
      type: 'unauthenticated',
      hasSmartcard: this.hasSmartcard,
    };
  }
}

export const UnauthenticatedSessionDataSchema: z.ZodSchema<UnauthenticatedSessionData> =
  UnauthenticatedSessionDataStructSchema.transform(
    (struct) => new UnauthenticatedSessionData(struct.hasSmartcard)
  ) as unknown as z.ZodSchema<UnauthenticatedSessionData>;

export type SessionData = UnauthenticatedSessionData | AuthenticatedSessionData;

export const SessionDataSchema: z.ZodSchema<SessionData> = z.union([
  UnauthenticatedSessionDataSchema,
  AuthenticatedSessionDataSchema,
]);
