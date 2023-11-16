import { Optional, assert, iter, typedAs } from '@votingworks/basics';
import { Byte } from '@votingworks/types';
import { Buffer } from 'buffer';
import * as fs from 'fs';
import { CardStatus, CheckPinResponse } from '../card';
import {
  CommonAccessCardCompatibleCard,
  CommonAccessCardDetails,
} from './common_access_card_api';

/**
 * The path of the file underlying a MockFileCard
 */
export const MOCK_FILE_PATH = '/tmp/mock-cac-file-card.json';

/**
 * A x509 key certificate pair
 */
export interface CertificatePair {
  key: Buffer;
  certificate: Buffer;
}

/**
 * The contents of the file underlying a MockFileCard
 */
export interface MockFileContents {
  cardStatus: CardStatus<CommonAccessCardDetails>;
  certificateMap: Map<Buffer, CertificatePair>;
  pin?: string;
}

/**
 * Converts a MockFileContents object into a Buffer
 */
export function serializeMockFileContents(
  mockFileContents: MockFileContents
): Buffer {
  const { cardStatus, certificateMap, pin } = mockFileContents;
  return Buffer.from(
    JSON.stringify({
      cardStatus,
      certificateMap: iter(certificateMap)
        .map(([objectId, { key, certificate }]) => [
          objectId.toString('hex'),
          key.toString('hex'),
          certificate.toString('hex'),
        ])
        .toArray(),
      pin,
    }),
    'utf-8'
  );
}

/**
 * Converts a Buffer created by serializeMockFileContents back into a MockFileContents object
 */
export function deserializeMockFileContents(file: Buffer): MockFileContents {
  const { cardStatus, certificateMap, pin } = JSON.parse(
    file.toString('utf-8')
  );
  return {
    cardStatus,
    certificateMap: new Map(
      typedAs<Array<[string, string, string]>>(certificateMap).map(
        ([objectId, key, certificate]) => [
          Buffer.from(objectId, 'hex'),
          {
            key: Buffer.from(key, 'hex'),
            certificate: Buffer.from(certificate, 'hex'),
          },
        ]
      )
    ),
    pin,
  };
}

function writeToMockFile(mockFileContents: MockFileContents): void {
  fs.writeFileSync(MOCK_FILE_PATH, serializeMockFileContents(mockFileContents));
}

/**
 * Mocks card actions by updating the file underlying a MockFileCard
 */
export const mockCard = writeToMockFile;

function initializeMockFile() {
  writeToMockFile({
    cardStatus: {
      status: 'no_card',
    },
    certificateMap: new Map(),
  });
}

/**
 * A helper for readFromMockFile. Returns undefined if the mock file doesn't exist or can't be
 * parsed.
 */
function readFromMockFileHelper(): Optional<MockFileContents> {
  if (!fs.existsSync(MOCK_FILE_PATH)) {
    return undefined;
  }
  const file = fs.readFileSync(MOCK_FILE_PATH);
  try {
    return deserializeMockFileContents(file);
  } catch {
    return undefined;
  }
}

/**
 * Reads and parses the contents of the file underlying a MockFileCard
 */
export function readFromMockFile(): MockFileContents {
  let mockFileContents = readFromMockFileHelper();
  if (!mockFileContents) {
    initializeMockFile();
    mockFileContents = readFromMockFileHelper();
    assert(mockFileContents !== undefined);
  }
  return mockFileContents;
}

/**
 * A mock implementation of the CAC card API that reads from and writes to a
 * file under the hood. Meant for local development and integration tests.
 *
 * Use ./scripts/mock-card in libs/auth/ to mock cards during local development.
 */
export class MockFileCard implements CommonAccessCardCompatibleCard {
  getCardStatus(): Promise<CardStatus<CommonAccessCardDetails>> {
    const { cardStatus } = readFromMockFile();
    return Promise.resolve(cardStatus);
  }

  checkPin(pin: string): Promise<CheckPinResponse> {
    const mockFileContents = readFromMockFile();
    const { cardStatus } = mockFileContents;
    assert(
      cardStatus.status === 'ready' && cardStatus.cardDetails !== undefined
    );
    if (pin === mockFileContents.pin) {
      return Promise.resolve({ response: 'correct' });
    }
    return Promise.resolve({
      response: 'incorrect',
      numIncorrectPinAttempts: 0,
    });
  }

  generateSignature(
    message: Buffer,
    options: { privateKeyId: Byte; pin?: string }
  ): Promise<Buffer> {
    throw new Error('Method not implemented.');
  }

  getCertificate(options: { objectId: Buffer }): Promise<Buffer> {
    const { objectId } = options;
    const { certificateMap } = readFromMockFile();
    const certificatePair = certificateMap.get(objectId);
    if (!certificatePair) {
      return Promise.reject(
        new Error(`No certificate found for objectId ${objectId}`)
      );
    }
    return Promise.resolve(certificatePair.certificate);
  }

  disconnect(): Promise<void> {
    return Promise.resolve();
  }
}
