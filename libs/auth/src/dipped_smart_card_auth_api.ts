import { Result } from '@votingworks/basics';
import {
  DippedSmartCardAuth,
  ElectionManagerUser,
  NumIncorrectPinAttemptsAllowedBeforeCardLockout,
  OverallSessionTimeLimitHours,
  PollWorkerUser,
  StartingCardLockoutDurationSeconds,
  SystemAdministratorUser,
} from '@votingworks/types';
import { SmartCardError } from './error';

/**
 * The API for a dipped smart card auth instance, "dipped" meaning that the card needs to be
 * inserted and removed from the card reader for the user to be authenticated
 */
export interface DippedSmartCardAuthApi {
  getAuthStatus(
    machineState: DippedSmartCardAuthMachineState
  ): Promise<Result<DippedSmartCardAuth.AuthStatus, SmartCardError>>;

  checkPin(
    machineState: DippedSmartCardAuthMachineState,
    input: { pin: string }
  ): Promise<Result<void, SmartCardError>>;
  logOut(machineState: DippedSmartCardAuthMachineState): Promise<void>;
  updateSessionExpiry(
    machineState: DippedSmartCardAuthMachineState,
    input: { sessionExpiresAt: Date }
  ): Promise<Result<void, SmartCardError>>;

  programCard(
    machineState: DippedSmartCardAuthMachineState,
    input:
      | { userRole: SystemAdministratorUser['role'] }
      | { userRole: ElectionManagerUser['role'] }
      | { userRole: PollWorkerUser['role'] }
  ): Promise<Result<{ pin?: string }, SmartCardError>>;
  unprogramCard(
    machineState: DippedSmartCardAuthMachineState
  ): Promise<Result<void, SmartCardError>>;
}

/**
 * Configuration parameters for a dipped smart card auth instance
 */
export interface DippedSmartCardAuthConfig {
  allowElectionManagersToAccessUnconfiguredMachines?: boolean;
}

/**
 * Machine state that the consumer is responsible for providing
 */
export interface DippedSmartCardAuthMachineState {
  arePollWorkerCardPinsEnabled?: boolean;
  electionHash?: string;
  jurisdiction?: string;
  numIncorrectPinAttemptsAllowedBeforeCardLockout?: NumIncorrectPinAttemptsAllowedBeforeCardLockout;
  overallSessionTimeLimitHours?: OverallSessionTimeLimitHours;
  startingCardLockoutDurationSeconds?: StartingCardLockoutDurationSeconds;
}
