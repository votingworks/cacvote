import { Buffer } from 'buffer';
import {
  Byte,
  ElectionManagerUser,
  PollWorkerUser,
  RaveVoterUser,
  SystemAdministratorUser,
} from '@votingworks/types';
import { throwIllegalValue } from '@votingworks/basics';

interface SystemAdministratorCardDetails {
  user: SystemAdministratorUser;
  numIncorrectPinAttempts?: number;
}

interface ElectionManagerCardDetails {
  user: ElectionManagerUser;
  numIncorrectPinAttempts?: number;
}

interface PollWorkerCardDetails {
  user: PollWorkerUser;
  numIncorrectPinAttempts?: number;

  /**
   * Unlike system administrator and election manager cards, which always have PINs, poll worker
   * cards by default don't have PINs but can if the relevant system setting is enabled.
   */
  hasPin: boolean;
}

interface RaveVoterCardDetails {
  user: RaveVoterUser;
  numIncorrectPinAttempts?: number;
}

/**
 * Details about a programmed card
 */
export type CardDetails =
  | SystemAdministratorCardDetails
  | ElectionManagerCardDetails
  | PollWorkerCardDetails
  | RaveVoterCardDetails;

/**
 * A CardDetails type guard
 */
export function areSystemAdministratorCardDetails(
  cardDetails: CardDetails
): cardDetails is SystemAdministratorCardDetails {
  return cardDetails.user.role === 'system_administrator';
}

/**
 * A CardDetails type guard
 */
export function areElectionManagerCardDetails(
  cardDetails: CardDetails
): cardDetails is ElectionManagerCardDetails {
  return cardDetails.user.role === 'election_manager';
}

/**
 * A CardDetails type guard
 */
export function arePollWorkerCardDetails(
  cardDetails: CardDetails
): cardDetails is PollWorkerCardDetails {
  return cardDetails.user.role === 'poll_worker';
}

/**
 * A CardDetails type guard
 */
export function areRaveVoterCardDetails(
  cardDetails: CardDetails
): cardDetails is RaveVoterCardDetails {
  return cardDetails.user.role === 'rave_voter';
}

/**
 * Get the jurisdiction of a card, if any.
 */
export function getUserJurisdiction(
  user: CardDetails['user']
): string | undefined {
  switch (user.role) {
    case 'system_administrator':
    case 'election_manager':
    case 'poll_worker':
      return user.jurisdiction;

    case 'rave_voter':
      return undefined;

    /* istanbul ignore next */
    default:
      throwIllegalValue(user);
  }
}

interface CardStatusReady {
  status: 'ready';
  cardDetails?: CardDetails;
}

interface CardStatusNotReady {
  status: 'card_error' | 'no_card' | 'unknown_error';
}

/**
 * The status of a card in a card reader
 */
export type CardStatus = CardStatusReady | CardStatusNotReady;

interface CheckPinResponseCorrect {
  response: 'correct';
}

interface CheckPinResponseIncorrect {
  response: 'incorrect';
  numIncorrectPinAttempts: number;
}

/**
 * The response to a PIN check
 */
export type CheckPinResponse =
  | CheckPinResponseCorrect
  | CheckPinResponseIncorrect;

/**
 * The API for a smart card
 */
export interface Card {
  getCardStatus(): Promise<CardStatus>;

  checkPin(pin: string): Promise<CheckPinResponse>;
  generateSignature(
    message: Buffer,
    options: { privateKeyId: Byte; pin?: string }
  ): Promise<Buffer>;

  program(
    input:
      | { user: SystemAdministratorUser; pin: string }
      | { user: ElectionManagerUser; pin: string }
      | { user: PollWorkerUser; pin?: string }
  ): Promise<void>;
  unprogram(): Promise<void>;

  readData(): Promise<Buffer>;
  writeData(data: Buffer): Promise<void>;
  clearData(): Promise<void>;
}
