import {
  assert,
  asyncResultBlock,
  err,
  extractErrorMessage,
  ok,
  Optional,
  Result,
  throwIllegalValue,
} from '@votingworks/basics';
import {
  LogDispositionStandardTypes,
  LogEventId,
  Logger,
} from '@votingworks/logging';
import { DippedSmartCardAuth as DippedSmartCardAuthTypes } from '@votingworks/types';
import {
  BooleanEnvironmentVariableName,
  generatePin,
  isFeatureFlagEnabled,
} from '@votingworks/utils';

import {
  arePollWorkerCardDetails,
  Card,
  CardDetails,
  CardStatus,
  CheckPinResponse,
  getUserJurisdiction,
} from './card';
import {
  DippedSmartCardAuthApi,
  DippedSmartCardAuthConfig,
  DippedSmartCardAuthMachineState,
} from './dipped_smart_card_auth_api';
import { computeCardLockoutEndTime } from './lockout';
import { computeSessionEndTime } from './sessions';
import { SmartCardError, SmartCardErrorCode } from './error';

type CheckPinResponseExtended = CheckPinResponse | { response: 'error' };

type AuthAction =
  | { type: 'check_card_reader'; cardStatus: CardStatus }
  | { type: 'check_pin'; checkPinResponse: CheckPinResponseExtended }
  | { type: 'log_out' }
  | {
      type: 'update_session_expiry';
      sessionExpiresAt: Date;
    };

function cardStatusToProgrammableCard(
  machineState: DippedSmartCardAuthMachineState,
  cardStatus: CardStatus
): DippedSmartCardAuthTypes.ProgrammableCard {
  switch (cardStatus.status) {
    case 'card_error':
    case 'no_card':
    case 'unknown_error': {
      return { status: cardStatus.status };
    }
    case 'ready': {
      const { cardDetails } = cardStatus;
      const user = cardDetails?.user;
      return {
        status: 'ready',
        programmedUser:
          // If one jurisdiction somehow attains a card from another jurisdiction, treat it as
          // unprogrammed
          (user && getUserJurisdiction(user)) !== machineState.jurisdiction ||
          // If a poll worker card doesn't have a PIN but poll worker card PINs are enabled, treat
          // the card as unprogrammed. And vice versa. If a poll worker card does have a PIN but
          // poll worker card PINs are not enabled, also treat the card as unprogrammed.
          (cardDetails &&
            arePollWorkerCardDetails(cardDetails) &&
            cardDetails.hasPin !==
              Boolean(machineState.arePollWorkerCardPinsEnabled))
            ? undefined
            : user,
      };
    }
    /* istanbul ignore next: Compile-time check for completeness */
    default: {
      throwIllegalValue(cardStatus, 'status');
    }
  }
}

/**
 * Given a previous auth status and a new auth status following an auth status transition, infers
 * and logs the relevant auth event, if any
 */
async function logAuthEventIfNecessary(
  previousAuthStatus: DippedSmartCardAuthTypes.AuthStatus,
  newAuthStatus: DippedSmartCardAuthTypes.AuthStatus,
  logger: Logger
) {
  switch (previousAuthStatus.status) {
    case 'logged_out': {
      if (
        previousAuthStatus.reason === 'machine_locked' &&
        newAuthStatus.status === 'logged_out' &&
        newAuthStatus.reason !== 'machine_locked'
      ) {
        await logger.log(
          LogEventId.AuthLogin,
          newAuthStatus.cardUserRole ?? 'unknown',
          {
            disposition: LogDispositionStandardTypes.Failure,
            message: 'User failed login.',
            reason: newAuthStatus.reason,
          }
        );
      }
      return;
    }

    case 'checking_pin': {
      if (newAuthStatus.status === 'logged_out') {
        await logger.log(
          LogEventId.AuthPinEntry,
          previousAuthStatus.user.role,
          {
            disposition: LogDispositionStandardTypes.Failure,
            message: 'User canceled PIN entry.',
          }
        );
      } else if (newAuthStatus.status === 'checking_pin') {
        if (
          newAuthStatus.wrongPinEnteredAt &&
          newAuthStatus.wrongPinEnteredAt !==
            previousAuthStatus.wrongPinEnteredAt
        ) {
          await logger.log(
            LogEventId.AuthPinEntry,
            previousAuthStatus.user.role,
            {
              disposition: LogDispositionStandardTypes.Failure,
              message: 'User entered incorrect PIN.',
            }
          );
        }
      } else if (newAuthStatus.status === 'remove_card') {
        await logger.log(LogEventId.AuthPinEntry, newAuthStatus.user.role, {
          disposition: LogDispositionStandardTypes.Success,
          message: 'User entered correct PIN.',
        });
      }
      // PIN check errors are logged in checkPin, where we have access to the full error message
      return;
    }

    case 'remove_card': {
      if (newAuthStatus.status === 'logged_in') {
        await logger.log(LogEventId.AuthLogin, newAuthStatus.user.role, {
          disposition: LogDispositionStandardTypes.Success,
          message: 'User logged in.',
        });
      }
      return;
    }

    case 'logged_in': {
      if (newAuthStatus.status === 'logged_out') {
        await logger.log(LogEventId.AuthLogout, previousAuthStatus.user.role, {
          disposition: LogDispositionStandardTypes.Success,
          message: 'User logged out.',
        });
      }
      return;
    }

    /* istanbul ignore next: Compile-time check for completeness */
    default:
      throwIllegalValue(previousAuthStatus, 'status');
  }
}

/**
 * The implementation of the dipped smart card auth API
 */
export class DippedSmartCardAuth implements DippedSmartCardAuthApi {
  private authStatus: DippedSmartCardAuthTypes.AuthStatus;
  private readonly card: Card;
  private readonly config: DippedSmartCardAuthConfig;
  private readonly logger: Logger;

  constructor(input: {
    card: Card;
    config: DippedSmartCardAuthConfig;
    logger: Logger;
  }) {
    this.authStatus = DippedSmartCardAuthTypes.DEFAULT_AUTH_STATUS;
    this.card = input.card;
    this.config = input.config;
    this.logger = input.logger;
  }

  async getAuthStatus(
    machineState: DippedSmartCardAuthMachineState
  ): Promise<Result<DippedSmartCardAuthTypes.AuthStatus, SmartCardError>> {
    return asyncResultBlock(async (ret) => {
      (await this.checkCardReaderAndUpdateAuthStatus(machineState)).okOrElse(
        ret
      );
      return this.authStatus;
    });
  }

  async checkPin(
    machineState: DippedSmartCardAuthMachineState,
    input: { pin: string }
  ): Promise<Result<void, SmartCardError>> {
    return asyncResultBlock(async (ret) => {
      (await this.checkCardReaderAndUpdateAuthStatus(machineState)).okOrElse(
        ret
      );
      if (this.isLockedOut()) {
        return;
      }

      let checkPinResult: Result<CheckPinResponseExtended, SmartCardError> =
        await this.card.checkPin(input.pin);
      if (checkPinResult.isErr()) {
        const error = checkPinResult.err();
        const userRole =
          'user' in this.authStatus ? this.authStatus.user.role : 'unknown';
        await this.logger.log(LogEventId.AuthPinEntry, userRole, {
          disposition: LogDispositionStandardTypes.Failure,
          message: `Error checking PIN: ${extractErrorMessage(error)}`,
        });
        checkPinResult = ok({ response: 'error' });
      }
      await this.updateAuthStatus(machineState, {
        type: 'check_pin',
        checkPinResponse: checkPinResult.okOrElse(ret),
      });
    });
  }

  async logOut(machineState: DippedSmartCardAuthMachineState): Promise<void> {
    await this.updateAuthStatus(machineState, { type: 'log_out' });
  }

  async updateSessionExpiry(
    machineState: DippedSmartCardAuthMachineState,
    input: { sessionExpiresAt: Date }
  ): Promise<Result<void, SmartCardError>> {
    return asyncResultBlock(async (ret) => {
      (await this.checkCardReaderAndUpdateAuthStatus(machineState)).okOrElse(
        ret
      );
      await this.updateAuthStatus(machineState, {
        type: 'update_session_expiry',
        sessionExpiresAt: input.sessionExpiresAt,
      });
    });
  }

  async programCard(
    machineState: DippedSmartCardAuthMachineState,
    input:
      | { userRole: 'system_administrator' }
      | { userRole: 'election_manager' }
      | { userRole: 'poll_worker' }
  ): Promise<Result<{ pin?: string }, SmartCardError>> {
    await this.logger.log(
      LogEventId.SmartCardProgramInit,
      'system_administrator',
      {
        message: 'Programming smart card...',
        programmedUserRole: input.userRole,
      }
    );
    const programResult = await this.programCardBase(machineState, input);

    if (programResult.isErr()) {
      const error = programResult.err();
      await this.logger.log(
        LogEventId.SmartCardProgramComplete,
        'system_administrator',
        {
          disposition: LogDispositionStandardTypes.Failure,
          message: `Error programming smart card: ${extractErrorMessage(
            error
          )}`,
          programmedUserRole: input.userRole,
        }
      );
      return err(
        SmartCardError(
          SmartCardErrorCode.UnknownError,
          'Error programming card'
        )
      );
    }

    return ok({ pin: programResult.ok() });
  }

  async unprogramCard(
    machineState: DippedSmartCardAuthMachineState
  ): Promise<Result<void, SmartCardError>> {
    const programmedUserRole =
      ('programmableCard' in this.authStatus &&
        'programmedUser' in this.authStatus.programmableCard &&
        /* istanbul ignore next */
        this.authStatus.programmableCard.programmedUser?.role) ||
      'unprogrammed';
    await this.logger.log(
      LogEventId.SmartCardUnprogramInit,
      'system_administrator',
      {
        message: 'Unprogramming smart card...',
        programmedUserRole,
      }
    );

    const unprogramResult = await this.unprogramCardBase(machineState);

    if (unprogramResult.isErr()) {
      const error = unprogramResult.err();
      await this.logger.log(
        LogEventId.SmartCardUnprogramComplete,
        'system_administrator',
        {
          disposition: LogDispositionStandardTypes.Failure,
          message: `Error unprogramming smart card: ${extractErrorMessage(
            error
          )}`,
          programmedUserRole,
        }
      );
      return err(
        SmartCardError(
          SmartCardErrorCode.UnknownError,
          'Error unprogramming card'
        )
      );
    }

    await this.logger.log(
      LogEventId.SmartCardUnprogramComplete,
      'system_administrator',
      {
        disposition: LogDispositionStandardTypes.Success,
        message: 'Successfully unprogrammed smart card.',
        previousProgrammedUserRole: programmedUserRole,
      }
    );
    return ok();
  }

  private async programCardBase(
    machineState: DippedSmartCardAuthMachineState,
    input:
      | { userRole: 'system_administrator' }
      | { userRole: 'election_manager' }
      | { userRole: 'poll_worker' }
  ): Promise<Result<Optional<string>, SmartCardError>> {
    return asyncResultBlock(async (ret) => {
      (await this.checkCardReaderAndUpdateAuthStatus(machineState)).okOrElse(
        ret
      );
      if (this.authStatus.status !== 'logged_in') {
        return err(
          SmartCardError(
            SmartCardErrorCode.AuthenticationError,
            'User is not logged in'
          )
        );
      }
      if (this.authStatus.user.role !== 'system_administrator') {
        return err(
          SmartCardError(
            SmartCardErrorCode.AuthorizationError,
            'User is not a system administrator'
          )
        );
      }

      const { arePollWorkerCardPinsEnabled, electionHash, jurisdiction } =
        machineState;
      assert(jurisdiction !== undefined);
      const pin = generatePin();
      switch (input.userRole) {
        case 'system_administrator': {
          (
            await this.card.program({
              user: { role: 'system_administrator', jurisdiction },
              pin,
            })
          ).okOrElse(ret);
          return pin;
        }
        case 'election_manager': {
          assert(electionHash !== undefined);
          (
            await this.card.program({
              user: { role: 'election_manager', jurisdiction, electionHash },
              pin,
            })
          ).okOrElse(ret);
          return pin;
        }
        case 'poll_worker': {
          assert(electionHash !== undefined);
          if (arePollWorkerCardPinsEnabled) {
            (
              await this.card.program({
                user: { role: 'poll_worker', jurisdiction, electionHash },
                pin,
              })
            ).okOrElse(ret);
            return pin;
          }
          (
            await this.card.program({
              user: { role: 'poll_worker', jurisdiction, electionHash },
            })
          ).okOrElse(ret);
          return undefined;
        }
        /* istanbul ignore next: Compile-time check for completeness */
        default: {
          throwIllegalValue(input, 'userRole');
        }
      }
    });
  }

  private async unprogramCardBase(
    machineState: DippedSmartCardAuthMachineState
  ): Promise<Result<void, SmartCardError>> {
    return asyncResultBlock(async (ret) => {
      (await this.checkCardReaderAndUpdateAuthStatus(machineState)).okOrElse(
        ret
      );
      if (this.authStatus.status !== 'logged_in') {
        return err(
          SmartCardError(
            SmartCardErrorCode.AuthenticationError,
            'User is not logged in'
          )
        );
      }
      if (this.authStatus.user.role !== 'system_administrator') {
        return err(
          SmartCardError(
            SmartCardErrorCode.AuthorizationError,
            'User is not a system administrator'
          )
        );
      }

      (await this.card.unprogram()).okOrElse(ret);
    });
  }

  private async checkCardReaderAndUpdateAuthStatus(
    machineState: DippedSmartCardAuthMachineState
  ): Promise<Result<void, SmartCardError>> {
    return asyncResultBlock(async (ret) => {
      const cardStatus = (await this.card.getCardStatus()).okOrElse(ret);
      await this.updateAuthStatus(machineState, {
        type: 'check_card_reader',
        cardStatus,
      });
    });
  }

  private async updateAuthStatus(
    machineState: DippedSmartCardAuthMachineState,
    action: AuthAction
  ): Promise<void> {
    const previousAuthStatus = this.authStatus;
    this.authStatus = this.determineNewAuthStatus(machineState, action);
    await logAuthEventIfNecessary(
      previousAuthStatus,
      this.authStatus,
      this.logger
    );
  }

  private determineNewAuthStatus(
    machineState: DippedSmartCardAuthMachineState,
    action: AuthAction
  ): DippedSmartCardAuthTypes.AuthStatus {
    const currentAuthStatus: DippedSmartCardAuthTypes.AuthStatus =
      this.authStatus.status === 'logged_in' &&
      new Date() >= this.authStatus.sessionExpiresAt
        ? { status: 'logged_out', reason: 'machine_locked' }
        : this.authStatus;

    switch (action.type) {
      case 'check_card_reader': {
        switch (currentAuthStatus.status) {
          case 'logged_out': {
            switch (action.cardStatus.status) {
              // TODO: Consider an alternative screen on the frontend for unknown errors
              case 'no_card':
              case 'unknown_error': {
                return { status: 'logged_out', reason: 'machine_locked' };
              }
              case 'card_error': {
                return { status: 'logged_out', reason: 'card_error' };
              }
              case 'ready': {
                const { cardDetails } = action.cardStatus;
                const validationResult = this.validateCard(
                  machineState,
                  cardDetails
                );
                if (validationResult.isOk()) {
                  assert(cardDetails !== undefined);
                  const { user } = cardDetails;
                  assert(
                    user.role === 'system_administrator' ||
                      user.role === 'election_manager'
                  );
                  const skipPinEntry = isFeatureFlagEnabled(
                    BooleanEnvironmentVariableName.SKIP_PIN_ENTRY
                  );
                  return skipPinEntry
                    ? {
                        status: 'remove_card',
                        user,
                        sessionExpiresAt: computeSessionEndTime(machineState),
                      }
                    : {
                        status: 'checking_pin',
                        user,
                        lockedOutUntil: computeCardLockoutEndTime(
                          machineState,
                          cardDetails.numIncorrectPinAttempts
                        ),
                      };
                }
                return {
                  status: 'logged_out',
                  reason: validationResult.err(),
                  cardUserRole: cardDetails?.user.role,
                };
              }
              /* istanbul ignore next: Compile-time check for completeness */
              default: {
                return throwIllegalValue(action.cardStatus, 'status');
              }
            }
          }

          case 'checking_pin': {
            if (action.cardStatus.status === 'no_card') {
              return { status: 'logged_out', reason: 'machine_locked' };
            }
            return currentAuthStatus;
          }

          case 'remove_card': {
            if (action.cardStatus.status === 'no_card') {
              const { user, sessionExpiresAt } = currentAuthStatus;
              if (user.role === 'system_administrator') {
                return {
                  status: 'logged_in',
                  user,
                  sessionExpiresAt,
                  programmableCard: cardStatusToProgrammableCard(
                    machineState,
                    action.cardStatus
                  ),
                };
              }
              return { status: 'logged_in', user, sessionExpiresAt };
            }
            return currentAuthStatus;
          }

          case 'logged_in': {
            const { user } = currentAuthStatus;
            if (user.role === 'system_administrator') {
              return {
                ...currentAuthStatus,
                programmableCard: cardStatusToProgrammableCard(
                  machineState,
                  action.cardStatus
                ),
              };
            }
            return currentAuthStatus;
          }

          /* istanbul ignore next: Compile-time check for completeness */
          default: {
            return throwIllegalValue(currentAuthStatus, 'status');
          }
        }
      }

      case 'check_pin': {
        if (currentAuthStatus.status !== 'checking_pin') {
          return currentAuthStatus;
        }
        switch (action.checkPinResponse.response) {
          case 'correct': {
            return {
              status: 'remove_card',
              user: currentAuthStatus.user,
              sessionExpiresAt: computeSessionEndTime(machineState),
            };
          }
          case 'incorrect': {
            return {
              ...currentAuthStatus,
              error: undefined,
              lockedOutUntil: computeCardLockoutEndTime(
                machineState,
                action.checkPinResponse.numIncorrectPinAttempts
              ),
              wrongPinEnteredAt: new Date(),
            };
          }
          case 'error': {
            return { ...currentAuthStatus, error: true };
          }
          /* istanbul ignore next: Compile-time check for completeness */
          default: {
            return throwIllegalValue(action.checkPinResponse, 'response');
          }
        }
      }

      case 'log_out': {
        return { status: 'logged_out', reason: 'machine_locked' };
      }

      case 'update_session_expiry': {
        if (
          currentAuthStatus.status !== 'remove_card' &&
          currentAuthStatus.status !== 'logged_in'
        ) {
          return currentAuthStatus;
        }
        return {
          ...currentAuthStatus,
          sessionExpiresAt: action.sessionExpiresAt,
        };
      }

      /* istanbul ignore next: Compile-time check for completeness */
      default: {
        throwIllegalValue(action, 'type');
      }
    }
  }

  private validateCard(
    machineState: DippedSmartCardAuthMachineState,
    cardDetails?: CardDetails
  ): Result<void, DippedSmartCardAuthTypes.LoggedOut['reason']> {
    if (!cardDetails) {
      return err('invalid_user_on_card');
    }

    const { user } = cardDetails;

    if (
      machineState.jurisdiction &&
      getUserJurisdiction(user) !== machineState.jurisdiction
    ) {
      return err('invalid_user_on_card');
    }

    if (!['system_administrator', 'election_manager'].includes(user.role)) {
      return err('user_role_not_allowed');
    }

    if (user.role === 'election_manager') {
      if (!machineState.electionHash) {
        return this.config.allowElectionManagersToAccessUnconfiguredMachines
          ? ok()
          : err('machine_not_configured');
      }
      if (user.electionHash !== machineState.electionHash) {
        return err('election_manager_wrong_election');
      }
    }

    return ok();
  }

  private isLockedOut(): boolean {
    return Boolean(
      this.authStatus.status === 'checking_pin' &&
        this.authStatus.lockedOutUntil &&
        new Date() < this.authStatus.lockedOutUntil
    );
  }
}
