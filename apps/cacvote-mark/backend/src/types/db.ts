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
 * `ServerId` is the ID on the RAVE Server, not the ID in this backend.
 */
export type ServerId = NewType<Id, 'ServerId'>;

/**
 * Creates a new `ServerId`.
 */
export function ServerId(): ServerId {
  return uuid() as ServerId;
}

/**
 * `ClientId` is the ID on this backend, not the ID on the RAVE Server.
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

export interface JurisdictionRow {
  id: string;
  name: string;
  createdAt: string;
}

export interface Jurisdiction {
  /**
   * Database ID for a jurisdiction record.
   */
  id: ServerId;

  /**
   * Name of the jurisdiction.
   */
  name: string;

  /**
   * Date and time when the jurisdiction was created.
   */
  createdAt: DateTime;
}

export function deserializeJurisdiction(row: JurisdictionRow): Jurisdiction {
  return {
    id: row.id as ServerId,
    name: row.name,
    createdAt: DateTime.fromSQL(row.createdAt),
  };
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
   * ID for the jurisdiction that this election belongs to.
   */
  jurisdictionId: ServerId;

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
  jurisdictionId: string;
  definition: Buffer;
}

export function deserializeElection(row: ElectionRow): Election {
  return {
    id: row.id as ClientId,
    jurisdictionId: row.jurisdictionId as ServerId,
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
   * Common Access Card X509 certificate.
   */
  commonAccessCardCertificate: Buffer;

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

export interface PrintedBallotRow {
  id: string;
  serverId: string | null;
  clientId: string;
  machineId: string;
  commonAccessCardId: string;
  commonAccessCardCertificate: Buffer;
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
    commonAccessCardCertificate: row.commonAccessCardCertificate,
    registrationId: row.registrationId as ClientId,
    castVoteRecord: row.castVoteRecord,
    castVoteRecordSignature: row.castVoteRecordSignature,
    createdAt: DateTime.fromSQL(row.createdAt),
  };
}
