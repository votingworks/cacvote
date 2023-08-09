import { Buffer } from 'buffer';
import { Optional } from '@votingworks/basics';
import {
  BallotStyleId,
  ElectionDefinition,
  Id,
  IdSchema,
  NewType,
  PrecinctId,
  safeParseElectionDefinition,
} from '@votingworks/types';
import { v4 as uuid } from 'uuid';
import { DateTime } from 'luxon';
import { z } from 'zod';

/**
 * `ServerId` is the ID on the RAVE server, not the ID in this backend.
 */
export type ServerId = NewType<Id, 'ServerId'>;

/**
 * Creates a new `ServerId`.
 */
export function ServerId(): ServerId {
  return uuid() as ServerId;
}

/**
 * `ClientId` is the ID on this backend, not the ID on the RAVE server.
 */
export type ClientId = NewType<Id, 'ClientId'>;

/**
 * Creates a new `ClientId`.
 */
export function ClientId(): ClientId {
  return uuid() as ClientId;
}

/**
 * Schema for {@link ServerId}.
 */
export const ServerIdSchema = IdSchema as z.ZodSchema<ServerId>;

/**
 * Schema for {@link ClientId}.
 */
export const ClientIdSchema = IdSchema as z.ZodSchema<ClientId>;

export interface RegistrationRequest {
  /**
   * Database ID for a registration request record.
   */
  id: ClientId;

  /**
   * Server-side ID for this registration request record, if sync'ed.
   */
  serverId?: ServerId;

  /**
   * Client-side ID for this registration request record.
   */
  clientId: ClientId;

  /**
   * Machine ID for the machine that created this registration request record.
   */
  machineId: Id;

  /**
   * Common Access Card ID for the voter who created this registration request.
   */
  commonAccessCardId: Id;

  /**
   * Voter's given name, i.e. first name.
   */
  givenName: string;

  /**
   * Voter's family name, i.e. last name.
   */
  familyName: string;

  /**
   * Voter's address line 1.
   */
  addressLine1: string;

  /**
   * Voter's address line 2.
   */
  addressLine2?: string;

  /**
   * Voter's city.
   */
  city: string;

  /**
   * Voter's state.
   */
  state: string;

  /**
   * Voter's postal code.
   */
  postalCode: string;

  /**
   * State-issued ID number for the voter, e.g. driver's license number.
   */
  stateId: string;

  /**
   * Date and time when the voter registered for the election.
   */
  createdAt: DateTime;
}

export interface RegistrationRequestRow {
  id: string;
  serverId: string | null;
  clientId: string;
  machineId: string;
  commonAccessCardId: string;
  givenName: string;
  familyName: string;
  addressLine1: string;
  addressLine2: string | null;
  city: string;
  state: string;
  postalCode: string;
  stateId: string;
  createdAt: string;
}

export interface Election {
  /**
   * Database ID for an election record.
   */
  id: ClientId;

  /**
   * Server-side ID for this election record, if sync'ed.
   */
  serverId?: ServerId;

  /**
   * Client-side ID for this election record.
   */
  clientId: ClientId;

  /**
   * Machine ID for the machine that created this election record.
   */
  machineId: Id;

  /**
   * Election data.
   */
  definition: Buffer;

  /**
   * Election definition.
   */
  electionDefinition: ElectionDefinition;
}

export interface ElectionRow {
  id: string;
  serverId: string | null;
  clientId: string;
  machineId: string;
  definition: Buffer;
}

export function deserializeElection(row: ElectionRow): Election {
  return {
    id: row.id as ClientId,
    serverId: row.serverId ? (row.serverId as ServerId) : undefined,
    clientId: row.clientId as ClientId,
    machineId: row.machineId,
    definition: row.definition,
    electionDefinition: safeParseElectionDefinition(
      row.definition.toString('utf-8')
    ).unsafeUnwrap(),
  };
}

export interface Registration {
  /**
   * Database ID for an election registration record.
   */
  id: ClientId;

  /**
   * Server-side ID for this registration record, if sync'ed.
   */
  serverId?: ServerId;

  /**
   * Client-side ID for this registration record.
   */
  clientId: ClientId;

  /**
   * Machine ID for the machine that created this registration record.
   */
  machineId: Id;

  /**
   * Common Access Card ID for the voter this registration belongs to.
   */
  commonAccessCardId: Id;

  /**
   * Database ID for the voter's registration request record.
   */
  registrationRequestId: ClientId;

  /**
   * Database ID for a the election record associated with this voter
   * registration.
   */
  electionId: ClientId;

  /**
   * Precinct ID for the voter's precinct.
   */
  precinctId: PrecinctId;

  /**
   * Ballot style ID for the voter's ballot style.
   */
  ballotStyleId: BallotStyleId;

  /**
   * Date and time when the voter registered for the election.
   */
  createdAt: DateTime;
}

export interface RegistrationRow {
  id: string;
  serverId: string | null;
  clientId: string;
  machineId: string;
  commonAccessCardId: string;
  registrationRequestId: string;
  electionId: string;
  precinctId: string;
  ballotStyleId: string;
  createdAt: string;
}

export interface PrintedBallot {
  /**
   * Database ID for a ballot record.
   */
  id: ClientId;

  /**
   * Server-side ID for this registration record, if sync'ed.
   */
  serverId?: ServerId;

  /**
   * Client-side ID for this registration record.
   */
  clientId: ClientId;

  /**
   * Machine ID for the machine that created this registration record.
   */
  machineId: Id;

  /**
   * Common Access Card ID for the voter who created this ballot.
   */
  commonAccessCardId: Id;

  /**
   * Database ID for the associated registration record.
   */
  registrationId: ClientId;

  /**
   * Votes cast by the voter.
   */
  castVoteRecord: Buffer;

  /**
   * Signature of the cast vote record.
   */
  castVoteRecordSignature: Buffer;

  /**
   * Date and time when the voter cast their votes.
   */
  createdAt: DateTime;
}

export interface ScannedBallotRow {
  id: string;
  serverId: string | null;
  clientId: string;
  machineId: string;
  electionId: string;
  castVoteRecord: Buffer;
  createdAt: string;
}

export function deserializeScannedBallot(row: ScannedBallotRow): ScannedBallot {
  return {
    id: row.id as ClientId,
    serverId: (row.serverId ?? undefined) as Optional<ServerId>,
    clientId: row.clientId as ClientId,
    machineId: row.machineId,
    electionId: row.electionId as ClientId,
    castVoteRecord: row.castVoteRecord,
    createdAt: DateTime.fromSQL(row.createdAt),
  };
}

export interface ScannedBallot {
  /**
   * Database ID for a ballot record.
   */
  id: ClientId;

  /**
   * Server-side ID for this registration record, if sync'ed.
   */
  serverId?: ServerId;

  /**
   * Client-side ID for this registration record.
   */
  clientId: ClientId;

  /**
   * Machine ID for the machine that created this registration record.
   */
  machineId: Id;

  /**
   * Database ID for the associated election record.
   */
  electionId: ClientId;

  /**
   * Votes cast by the voter.
   */
  castVoteRecord: Buffer;

  /**
   * Date and time when the voter cast their votes.
   */
  createdAt: DateTime;
}

export interface PrintedBallotRow {
  id: string;
  serverId: string | null;
  clientId: string;
  machineId: string;
  commonAccessCardId: string;
  registrationId: string;
  castVoteRecord: Buffer;
  castVoteRecordSignature: Buffer;
  createdAt: string;
}

export function deserializePrintedBallot(row: PrintedBallotRow): PrintedBallot {
  return {
    id: row.id as ClientId,
    serverId: (row.serverId ?? undefined) as Optional<ServerId>,
    clientId: row.clientId as ClientId,
    machineId: row.machineId,
    // because these are just strings of digits, sqlite may return them as
    // numbers, so we have to convert them back to strings
    commonAccessCardId: row.commonAccessCardId.toString(),
    registrationId: row.registrationId as ClientId,
    castVoteRecord: row.castVoteRecord,
    castVoteRecordSignature: row.castVoteRecordSignature,
    createdAt: DateTime.fromSQL(row.createdAt),
  };
}

export interface ServerSyncAttempt {
  /**
   * Database ID for a server sync attempt record.
   */
  id: ClientId;

  /**
   * Creator for the user who initiated the server sync attempt.
   */
  creator: string;

  /**
   * Trigger type for the server sync attempt.
   */
  trigger: string;

  /**
   * Status message for the server sync attempt.
   */
  statusMessage: string;

  /**
   * Date and time when the server sync attempt was made.
   */
  createdAt: DateTime;

  /**
   * Whether or not the server sync attempt was successful.
   */
  success?: boolean;

  /**
   * Date and time when the server sync attempt was completed.
   */
  completedAt?: DateTime;
}

export interface ServerSyncAttemptRow {
  id: string;
  creator: string;
  trigger: string;
  statusMessage: string;
  success: 0 | 1 | null;
  createdAt: string;
  completedAt: string | null;
}

export function deserializeRegistrationRequest(
  row: RegistrationRequestRow
): RegistrationRequest {
  return {
    id: row.id as ClientId,
    serverId: (row.serverId ?? undefined) as Optional<ServerId>,
    clientId: row.clientId as ClientId,
    machineId: row.machineId,
    // because these are just strings of digits, sqlite may return them as
    // numbers, so we have to convert them back to strings
    commonAccessCardId: row.commonAccessCardId.toString(),
    givenName: row.givenName,
    familyName: row.familyName,
    addressLine1: row.addressLine1,
    addressLine2: row.addressLine2 ?? undefined,
    city: row.city,
    state: row.state,
    postalCode: row.postalCode,
    stateId: row.stateId,
    createdAt: DateTime.fromSQL(row.createdAt),
  };
}

export function deserializeRegistration(row: RegistrationRow): Registration {
  return {
    id: row.id as ClientId,
    serverId: (row.serverId ?? undefined) as Optional<ServerId>,
    clientId: row.clientId as ClientId,
    machineId: row.machineId,
    // because these are just strings of digits, sqlite may return them as
    // numbers, so we have to convert them back to strings
    commonAccessCardId: row.commonAccessCardId.toString(),
    registrationRequestId: row.registrationRequestId as ClientId,
    electionId: row.electionId as ClientId,
    precinctId: row.precinctId,
    ballotStyleId: row.ballotStyleId,
    createdAt: DateTime.fromSQL(row.createdAt),
  };
}

export function deserializeServerSyncAttempt(
  row: ServerSyncAttemptRow
): ServerSyncAttempt {
  return {
    id: row.id as ClientId,
    creator: row.creator,
    trigger: row.trigger,
    statusMessage: row.statusMessage,
    createdAt: DateTime.fromSQL(row.createdAt),
    success: row.success === 1 ? true : row.success === 0 ? false : undefined,
    completedAt: row.completedAt
      ? DateTime.fromSQL(row.completedAt)
      : undefined,
  };
}
