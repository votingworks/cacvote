import { assert, throwIllegalValue } from '@votingworks/basics';
import { Byte } from '@votingworks/types';
import { Buffer } from 'buffer';
import { sha256 } from 'js-sha256';
import { v4 as uuid } from 'uuid';

import {
  CardCommand,
  constructTlv,
  parseTlv,
  ResponseApduError,
  SELECT,
} from './apdu';
import {
  Card,
  CardStatus,
  CheckPinResponse,
  CommonAccessCardDetails,
} from './card';
import { CardReader } from './card_reader';
import { parseCardDetailsFromCert } from './certs';
import {
  certDerToPem,
  extractPublicKeyFromCert,
  verifySignature,
} from './openssl';
import {
  construct8BytePinBuffer,
  GENERAL_AUTHENTICATE,
  GET_DATA,
  isIncorrectPinStatusWord,
  numRemainingPinAttemptsFromIncorrectPinStatusWord,
  pivDataObjectId,
  PUT_DATA,
  VERIFY,
} from './piv';

/**
 * The OpenFIPS201 applet ID
 */
export const OPEN_FIPS_201_AID = 'a000000308000010000100';

/**
 * The max number of incorrect PIN attempts before the card is completely locked and needs to be
 * reprogrammed. 15 is the max value supported by OpenFIPS201.
 */
export const MAX_NUM_INCORRECT_PIN_ATTEMPTS = 15;

/**
 * The card's DoD-issued cert.
 */
export const CARD_CERT = {
  OBJECT_ID: pivDataObjectId(0x0a),
  PRIVATE_KEY_ID: 0x9c,
} as const;

/**
 * An implementation of the card API that uses a Java Card running our fork of the OpenFIPS201
 * applet (https://github.com/votingworks/openfips201) and X.509 certs. The implementation takes
 * inspiration from the NIST PIV standard but diverges where PIV doesn't suit our needs.
 */
export class JavaCard implements Card {
  private readonly cardReader: CardReader;
  private cardStatus: CardStatus;

  constructor() {
    this.cardStatus = { status: 'no_card' };

    this.cardReader = new CardReader({
      onReaderStatusChange: async (readerStatus) => {
        switch (readerStatus) {
          case 'no_card':
          case 'no_card_reader': {
            this.cardStatus = { status: 'no_card' };
            return;
          }
          case 'card_error': {
            this.cardStatus = { status: 'card_error' };
            return;
          }
          case 'unknown_error': {
            this.cardStatus = { status: 'unknown_error' };
            return;
          }
          case 'ready': {
            const cardDetails = await this.readCardDetails();
            this.cardStatus = { status: 'ready', cardDetails };
            return;
          }
          /* istanbul ignore next: Compile-time check for completeness */
          default: {
            throwIllegalValue(readerStatus);
          }
        }
      },
    });
  }

  private generateChallenge(): string {
    return `VotingWorks/${new Date().toISOString()}/${uuid()}`;
  }

  async getCardStatus(): Promise<CardStatus> {
    return Promise.resolve(this.cardStatus);
  }

  async checkPin(pin: string): Promise<CheckPinResponse> {
    await this.selectApplet();

    const cardVxAdminCert = await this.getCertificate({
      objectId: CARD_CERT.OBJECT_ID,
    });
    try {
      // Verify that the card has a private key that corresponds to the public key in the card
      // VxAdmin cert
      await this.verifyCardPrivateKey(
        CARD_CERT.PRIVATE_KEY_ID,
        cardVxAdminCert,
        pin
      );
    } catch (error) {
      if (
        error instanceof ResponseApduError &&
        isIncorrectPinStatusWord(error.statusWord())
      ) {
        const numIncorrectPinAttempts =
          MAX_NUM_INCORRECT_PIN_ATTEMPTS -
          numRemainingPinAttemptsFromIncorrectPinStatusWord(error.statusWord());
        if (this.cardStatus.status === 'ready' && this.cardStatus.cardDetails) {
          this.cardStatus = {
            status: 'ready',
            cardDetails: {
              ...this.cardStatus.cardDetails,
              numIncorrectPinAttempts,
            },
          };
        }
        return {
          response: 'incorrect',
          numIncorrectPinAttempts,
        };
      }
      throw error;
    }
    if (this.cardStatus.status === 'ready' && this.cardStatus.cardDetails) {
      this.cardStatus = {
        status: 'ready',
        cardDetails: {
          ...this.cardStatus.cardDetails,
          numIncorrectPinAttempts: 0,
        },
      };
    }
    return { response: 'correct' };
  }

  /**
   * Reads the card details, performing various forms of verification along the way. Throws an
   * error if any verification fails (this includes the case that the card is simply unprogrammed).
   */
  private async readCardDetails(): Promise<CommonAccessCardDetails> {
    await this.selectApplet();
    const cert = await this.getCertificate({ objectId: CARD_CERT.OBJECT_ID });
    return parseCardDetailsFromCert(cert);
  }

  /**
   * Selects the OpenFIPS201 applet
   */
  private async selectApplet(): Promise<void> {
    await this.cardReader.transmit(
      new CardCommand({
        ins: SELECT.INS,
        p1: SELECT.P1,
        p2: SELECT.P2,
        data: Buffer.from(OPEN_FIPS_201_AID, 'hex'),
      })
    );
  }

  /**
   * Verifies that the specified card private key corresponds to the public key in the provided
   * cert by 1) having the private key sign a "challenge" and 2) using the public key to verify the
   * generated signature.
   *
   * A PIN must be provided if the private key is PIN-gated.
   */
  private async verifyCardPrivateKey(
    privateKeyId: Byte,
    cert: Buffer,
    pin?: string
  ): Promise<void> {
    // Have the private key sign a "challenge"
    const challenge = this.generateChallenge();
    const challengeBuffer = Buffer.from(challenge, 'utf-8');
    const challengeSignature = await this.generateSignature(challengeBuffer, {
      privateKeyId,
      pin,
    });

    // Use the cert's public key to verify the generated signature
    const certPublicKey = await extractPublicKeyFromCert(cert);
    await verifySignature({
      message: challengeBuffer,
      messageSignature: challengeSignature,
      publicKey: certPublicKey,
    });
  }

  /**
   * Signs a message using the specified private key. A PIN must be provided if
   * the private key is PIN-gated.
   */
  async generateSignature(
    message: Buffer,
    { privateKeyId, pin }: { privateKeyId: Byte; pin?: string }
  ): Promise<Buffer> {
    if (pin) {
      await this.checkPinInternal(pin);
    }

    const challengeHash = Buffer.from(sha256(message), 'hex');
    const asn1Sha256MagicValue = Buffer.from([
      0x30, 0x31, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03,
      0x04, 0x02, 0x01, 0x05, 0x00, 0x04, 0x20,
    ]);

    const allFsPadding = Buffer.alloc(256 - 19 - 2 - 1 - 32, 0xff);

    // now we pad
    const paddedMessage = Buffer.concat([
      Buffer.from([0, 1]),
      allFsPadding,
      Buffer.from([0]),
      asn1Sha256MagicValue,
      challengeHash,
    ]);
    assert(paddedMessage.length === 256);

    const generalAuthenticateResponse = await this.cardReader.transmit(
      new CardCommand({
        ins: GENERAL_AUTHENTICATE.INS,
        p1: 0x07, // CRYPTOGRAPHIC_ALGORITHM_IDENTIFIER.ECC256,
        p2: privateKeyId,
        data: constructTlv(
          GENERAL_AUTHENTICATE.DYNAMIC_AUTHENTICATION_TEMPLATE_TAG,

          Buffer.concat([
            constructTlv(GENERAL_AUTHENTICATE.CHALLENGE_TAG, paddedMessage),
            constructTlv(GENERAL_AUTHENTICATE.RESPONSE_TAG, Buffer.from([])),
          ])
        ),
      })
    );

    // why are we trimming 8 here instead of 4, and why were we trimming 4 before? Is this an ECC vs. RSA formatting?
    return generalAuthenticateResponse.subarray(8); // Trim metadata
  }

  /**
   * Retrieves a cert in PEM format.
   */
  async getCertificate(options: { objectId: Buffer }): Promise<Buffer> {
    const data = await this.getData(options.objectId);
    const certTlv = data.subarray(0, -5); // Trim metadata
    const [, , certInDerFormat] = parseTlv(PUT_DATA.CERT_TAG, certTlv);
    return await certDerToPem(certInDerFormat);
  }

  /**
   * The underlying call for checking a PIN. Throws a ResponseApduError with an incorrect-PIN
   * status word if an incorrect PIN is entered. Named checkPinInternal to avoid a conflict with
   * the public checkPin method.
   */
  private async checkPinInternal(pin: string): Promise<void> {
    await this.cardReader.transmit(
      new CardCommand({
        ins: VERIFY.INS,
        p1: VERIFY.P1_VERIFY,
        p2: VERIFY.P2_PIN,
        data: construct8BytePinBuffer(pin),
      })
    );
  }

  private async getData(objectId: Buffer): Promise<Buffer> {
    const dataTlv = await this.cardReader.transmit(
      new CardCommand({
        ins: GET_DATA.INS,
        p1: GET_DATA.P1,
        p2: GET_DATA.P2,
        data: constructTlv(GET_DATA.TAG_LIST_TAG, objectId),
      })
    );
    const [, , data] = parseTlv(PUT_DATA.DATA_TAG, dataTlv);
    return data;
  }

  /**
   * Disconnects the card so that it can be reconnected to, through a new JavaCard instance
   */
  async disconnect(): Promise<void> {
    await this.cardReader.disconnectCard();
  }
}
