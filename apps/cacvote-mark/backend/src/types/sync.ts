import { Buffer } from 'buffer';
import { z } from 'zod';
import {
  BallotStyleId,
  BallotStyleIdSchema,
  Id,
  NewType,
  PrecinctId,
  PrecinctIdSchema,
} from '@votingworks/types';
import { DateTime } from 'luxon';
import { ClientId, ClientIdSchema, ServerId, ServerIdSchema } from './db';

export type Base64String = NewType<string, 'Base64String'>;
export const Base64StringSchema = z.string().superRefine((value, ctx) => {
  try {
    Buffer.from(value, 'base64');
    return true;
  } catch (error) {
    ctx.addIssue({
      code: z.ZodIssueCode.custom,
      path: ctx.path,
      message: `invalid base64 string: ${(error as Error).message}`,
    });
  }
}) as unknown as z.ZodSchema<Base64String>;

export const Base64Buffer = z
  .string()
  .transform((value) =>
    Buffer.from(value, 'base64')
  ) as unknown as z.ZodSchema<Buffer>;

export const DateTimeSchema = z
  .string()
  .transform((value) =>
    DateTime.fromISO(value)
  ) as unknown as z.ZodSchema<DateTime>;

export interface RegistrationRequestInput {
  clientId: ClientId;
  machineId: Id;
  jurisdictionId: ServerId;
  commonAccessCardId: string;
  givenName: string;
  familyName: string;
}

export type RegistrationRequestOutput = RegistrationRequestInput & {
  serverId: ServerId;
  createdAt: DateTime;
};

export const RegistrationRequestOutputSchema: z.ZodSchema<RegistrationRequestOutput> =
  z.object({
    serverId: ServerIdSchema,
    clientId: ClientIdSchema,
    machineId: z.string(),
    jurisdictionId: ServerIdSchema,
    commonAccessCardId: z.string(),
    givenName: z.string(),
    familyName: z.string(),
    createdAt: DateTimeSchema,
  });

export interface RegistrationInput {
  clientId: ClientId;
  machineId: Id;
  commonAccessCardId: Id;
  registrationRequestId: ClientId;
  electionId: ClientId;
  precinctId: PrecinctId;
  ballotStyleId: BallotStyleId;
}

export interface RegistrationOutput {
  serverId: ServerId;
  clientId: ClientId;
  machineId: Id;
  commonAccessCardId: Id;
  jurisdictionId: ServerId;
  registrationRequestId: ServerId;
  electionId: ServerId;
  precinctId: PrecinctId;
  ballotStyleId: BallotStyleId;
  createdAt: DateTime;
}

export const RegistrationOutputSchema: z.ZodSchema<RegistrationOutput> =
  z.object({
    serverId: ServerIdSchema,
    clientId: ClientIdSchema,
    machineId: z.string(),
    commonAccessCardId: z.string(),
    jurisdictionId: ServerIdSchema,
    registrationRequestId: ServerIdSchema,
    electionId: ServerIdSchema,
    precinctId: PrecinctIdSchema,
    ballotStyleId: BallotStyleIdSchema,
    createdAt: DateTimeSchema,
  });

export interface PrintedBallotInput {
  clientId: ClientId;
  machineId: Id;
  commonAccessCardId: string;
  commonAccessCardCertificate: Base64String;
  registrationId: ClientId;
  castVoteRecord: Base64String;
}

export interface PrintedBallotOutput {
  serverId: ServerId;
  clientId: ClientId;
  machineId: Id;
  commonAccessCardId: string;
  commonAccessCardCertificate: Base64String;
  registrationId: ServerId;
  castVoteRecord: Base64String;
  castVoteRecordSignature: Base64String;
}

export const PrintedBallotOutputSchema: z.ZodSchema<PrintedBallotOutput> =
  z.object({
    serverId: ServerIdSchema,
    clientId: ClientIdSchema,
    machineId: z.string(),
    commonAccessCardId: z.string(),
    commonAccessCardCertificate: Base64StringSchema,
    registrationId: ServerIdSchema,
    castVoteRecord: Base64StringSchema,
    castVoteRecordSignature: Base64StringSchema,
  });

export interface AdminInput {
  machineId: string;
  commonAccessCardId: string;
  createdAt: DateTime;
}

export interface JurisdictionInput {
  name: string;
}

export interface JurisdictionOutput {
  id: ServerId;
  name: string;
  createdAt: DateTime;
}

export const JurisdictionOutputSchema: z.ZodSchema<JurisdictionOutput> =
  z.object({
    id: ServerIdSchema,
    name: z.string().nonempty(),
    createdAt: DateTimeSchema,
  });

export type AdminOutput = AdminInput;

export const AdminOutputSchema: z.ZodSchema<AdminOutput> = z.object({
  machineId: z.string(),
  commonAccessCardId: z.string(),
  createdAt: DateTimeSchema,
});

export interface ElectionInput {
  jurisdictionId: ServerId;
  clientId: ClientId;
  machineId: Id;
  definition: Base64String;
}

export type ElectionOutput = ElectionInput & {
  serverId: ServerId;
  createdAt: DateTime;
};

export const ElectionOutputSchema: z.ZodSchema<ElectionOutput> = z.object({
  jurisdictionId: ServerIdSchema,
  serverId: ServerIdSchema,
  clientId: ClientIdSchema,
  machineId: z.string(),
  definition: Base64StringSchema,
  createdAt: DateTimeSchema,
});

export interface RaveServerSyncInput {
  lastSyncedRegistrationRequestId?: ServerId;
  lastSyncedRegistrationId?: ServerId;
  lastSyncedElectionId?: ServerId;
  lastSyncedPrintedBallotId?: ServerId;
  jurisdictions?: JurisdictionInput[];
  registrationRequests?: RegistrationRequestInput[];
  elections?: ElectionInput[];
  registrations?: RegistrationInput[];
  printedBallots?: PrintedBallotInput[];
}

export interface RaveServerSyncOutput {
  jurisdictions: JurisdictionOutput[];
  admins: AdminOutput[];
  elections: ElectionOutput[];
  registrationRequests: RegistrationRequestOutput[];
  registrations: RegistrationOutput[];
  printedBallots: PrintedBallotOutput[];
}

export const RaveMarkSyncOutputSchema: z.ZodSchema<RaveServerSyncOutput> =
  z.object({
    jurisdictions: z.array(JurisdictionOutputSchema),
    admins: z.array(AdminOutputSchema),
    elections: z.array(ElectionOutputSchema),
    registrationRequests: z.array(RegistrationRequestOutputSchema),
    registrations: z.array(RegistrationOutputSchema),
    printedBallots: z.array(PrintedBallotOutputSchema),
  });
