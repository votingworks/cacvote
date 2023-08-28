import { Byte, RaveVoterUser } from '@votingworks/types';
import { Buffer } from 'buffer';

/**
 * Details about a Common Access Card.
 */
export interface CommonAccessCardDetails {
  user: RaveVoterUser;
  numIncorrectPinAttempts: number;
}

/**
 * Details about a programmed card
 */
export type CardDetails = CommonAccessCardDetails;

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
  getCertificate(options: { objectId: Buffer }): Promise<Buffer>;
}
