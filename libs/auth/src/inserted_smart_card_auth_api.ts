import { Buffer } from 'buffer';
import { z } from 'zod';
import { Optional, Result } from '@votingworks/basics';
import {
  BallotStyleId,
  Byte,
  InsertedSmartCardAuth,
  NumIncorrectPinAttemptsAllowedBeforeCardLockout,
  OverallSessionTimeLimitHours,
  PrecinctId,
  StartingCardLockoutDurationSeconds,
} from '@votingworks/types';
import { SmartCardError } from './error';

/**
 * The API for an inserted smart card auth instance, "inserted" meaning that the card needs to be
 * kept in the card reader for the user to remain authenticated
 */
export interface InsertedSmartCardAuthApi {
  getAuthStatus(
    machineState: InsertedSmartCardAuthMachineState
  ): Promise<Result<InsertedSmartCardAuth.AuthStatus, SmartCardError>>;

  checkPin(
    machineState: InsertedSmartCardAuthMachineState,
    input: { pin: string }
  ): Promise<Result<void, SmartCardError>>;
  generateSignature(
    message: Buffer,
    options: { privateKeyId: Byte; pin?: string }
  ): Promise<Result<Buffer, SmartCardError>>;
  getCertificate(options: {
    objectId: Buffer;
  }): Promise<Result<Buffer, SmartCardError>>;

  /**
   * Though logout is typically accomplished by removing the inserted card when using inserted
   * smart card auth, this method is still useful for clearing the session and re-requiring PIN
   * entry, e.g. after the inactive session time limit has been hit.
   */
  logOut(machineState: InsertedSmartCardAuthMachineState): Promise<void>;
  updateSessionExpiry(
    machineState: InsertedSmartCardAuthMachineState,
    input: { sessionExpiresAt: Date }
  ): Promise<Result<void, SmartCardError>>;

  startCardlessVoterSession(
    machineState: InsertedSmartCardAuthMachineState,
    input: { ballotStyleId: BallotStyleId; precinctId: PrecinctId }
  ): Promise<Result<void, SmartCardError>>;
  endCardlessVoterSession(
    machineState: InsertedSmartCardAuthMachineState
  ): Promise<void>;

  readCardData<T>(
    machineState: InsertedSmartCardAuthMachineState,
    input: { schema: z.ZodSchema<T> }
  ): Promise<Result<Optional<T>, SyntaxError | z.ZodError | SmartCardError>>;
  readCardDataAsString(
    machineState: InsertedSmartCardAuthMachineState
  ): Promise<Result<Optional<string>, SmartCardError>>;
  writeCardData<T>(
    machineState: InsertedSmartCardAuthMachineState,
    input: { data: T; schema: z.ZodSchema<T> }
  ): Promise<Result<void, SmartCardError>>;
  clearCardData(
    machineState: InsertedSmartCardAuthMachineState
  ): Promise<Result<void, SmartCardError>>;
}

/**
 * Configuration parameters for an inserted smart card auth instance
 */
export interface InsertedSmartCardAuthConfig {
  allowCardlessVoterSessions?: boolean;
  allowElectionManagersToAccessMachinesConfiguredForOtherElections?: boolean;
}

/**
 * Machine state that the consumer is responsible for providing
 */
export interface InsertedSmartCardAuthMachineState {
  arePollWorkerCardPinsEnabled?: boolean;
  electionHash?: string;
  jurisdiction?: string;
  numIncorrectPinAttemptsAllowedBeforeCardLockout?: NumIncorrectPinAttemptsAllowedBeforeCardLockout;
  overallSessionTimeLimitHours?: OverallSessionTimeLimitHours;
  startingCardLockoutDurationSeconds?: StartingCardLockoutDurationSeconds;
}
