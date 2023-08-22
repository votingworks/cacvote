import { Buffer } from 'buffer';
import newPcscLite from 'pcsclite';
import { promisify } from 'util';
import { Result, assert, asyncResultBlock, err, ok } from '@votingworks/basics';
import { isByte } from '@votingworks/types';

import {
  CardCommand,
  CommandApdu,
  GET_RESPONSE,
  MAX_APDU_LENGTH,
  ResponseApduError,
  STATUS_WORD,
} from './apdu';
import { SmartCardError, SmartCardErrorCode } from './error';

/**
 * A PCSC Lite instance
 */
export type PcscLite = ReturnType<typeof newPcscLite>;

interface ReaderReady {
  status: 'ready';
  transmit: (data: Buffer) => Promise<Buffer>;
}

interface ReaderNotReady {
  status: 'card_error' | 'no_card_reader' | 'no_card' | 'unknown_error';
}

type Reader = ReaderReady | ReaderNotReady;

/**
 * The status of the smart card reader
 */
export type ReaderStatus = Reader['status'];

/**
 * A on-change handler for reader status changes
 */
export type OnReaderStatusChange = (readerStatus: ReaderStatus) => void;

/**
 * A class for interfacing with a smart card reader, implemented using PCSC Lite
 */
export class CardReader {
  private readonly onReaderStatusChange: OnReaderStatusChange;
  private readonly pcscLite: PcscLite;
  private reader: Reader;

  constructor(input: { onReaderStatusChange: OnReaderStatusChange }) {
    this.onReaderStatusChange = input.onReaderStatusChange;
    this.pcscLite = newPcscLite();
    this.reader = { status: 'no_card_reader' };

    this.pcscLite.on('error', () => {
      this.updateReader({ status: 'unknown_error' });
    });

    this.pcscLite.on('reader', (reader) => {
      reader.on('error', () => {
        this.updateReader({ status: 'unknown_error' });
      });

      reader.on('status', (status) => {
        const isCardPresent = Boolean(
          // eslint-disable-next-line no-bitwise
          status.state & reader.SCARD_STATE_PRESENT
        );
        if (isCardPresent) {
          reader.connect(
            // Don't allow anyone else to access the card reader while this code is accessing it
            { share_mode: reader.SCARD_SHARE_EXCLUSIVE },
            (error, protocol) => {
              if (error) {
                this.updateReader({ status: 'card_error' });
                return;
              }
              const transmitPromisified = promisify(reader.transmit).bind(
                reader
              );
              this.updateReader({
                status: 'ready',
                transmit: (data: Buffer) =>
                  transmitPromisified(data, MAX_APDU_LENGTH, protocol),
              });
            }
          );
        } else {
          this.updateReader({ status: 'no_card' });
          reader.disconnect(/* istanbul ignore next */ () => undefined);
        }
      });

      reader.on('end', () => {
        this.updateReader({ status: 'no_card_reader' });
      });
    });
  }

  /**
   * Transmits command APDUs to a smart card. On success, returns response data. On error, throws.
   * Specifically throws a ResponseApduError when a response APDU with an error status word is
   * received.
   */
  async transmit(
    command: CardCommand
  ): Promise<Result<Buffer, SmartCardError>> {
    return asyncResultBlock(async (ret) => {
      const apdus = command.asCommandApdus();
      let data: Buffer = Buffer.from([]);
      let moreDataAvailable = false;
      let moreDataLength = 0x00;

      for (const [i, apdu] of apdus.entries()) {
        if (i < apdus.length - 1) {
          // APDUs before the last in a chain
          (await this.transmitHelper(apdu)).okOrElse(ret);
        } else {
          const response = (await this.transmitHelper(apdu)).okOrElse(ret);
          data = Buffer.concat([data, response.data]);
          moreDataAvailable = response.moreDataAvailable;
          moreDataLength = response.moreDataLength;
        }
      }

      while (moreDataAvailable) {
        const response = (
          await this.transmitHelper(
            new CommandApdu({
              ins: GET_RESPONSE.INS,
              p1: GET_RESPONSE.P1,
              p2: GET_RESPONSE.P2,
              rawData: Buffer.from([moreDataLength]),
            })
          )
        ).okOrElse(ret);
        data = Buffer.concat([data, response.data]);
        moreDataAvailable = response.moreDataAvailable;
        moreDataLength = response.moreDataLength;
      }

      return data;
    });
  }

  private async transmitHelper(apdu: CommandApdu): Promise<
    Result<
      {
        data: Buffer;
        moreDataAvailable: boolean;
        moreDataLength: number;
      },
      SmartCardError
    >
  > {
    if (this.reader.status !== 'ready') {
      return err(
        SmartCardError(
          SmartCardErrorCode.NoCardReader,
          `Reader not ready: ${this.reader.status}`
        )
      );
    }

    let response: Buffer;
    try {
      response = await this.reader.transmit(apdu.asBuffer());
    } catch (error) {
      return err(
        SmartCardError(
          SmartCardErrorCode.TransmitFailed,
          error instanceof Error ? error.message : String(error)
        )
      );
    }

    const data = response.subarray(0, -2);
    const [sw1, sw2] = response.subarray(-2);
    assert(sw1 !== undefined && sw2 !== undefined);
    assert(isByte(sw1) && isByte(sw2));
    if (sw1 === STATUS_WORD.SUCCESS.SW1 && sw2 === STATUS_WORD.SUCCESS.SW2) {
      return ok({ data, moreDataAvailable: false, moreDataLength: 0 });
    }
    if (sw1 === STATUS_WORD.SUCCESS_MORE_DATA_AVAILABLE.SW1) {
      return ok({ data, moreDataAvailable: true, moreDataLength: sw2 ?? 0 });
    }
    return err(
      SmartCardError(
        SmartCardErrorCode.ResponseError,
        new ResponseApduError([sw1, sw2])
      )
    );
  }

  private updateReader(reader: Reader): void {
    const readerStatusChange = this.reader.status !== reader.status;
    this.reader = reader;
    if (readerStatusChange) {
      this.onReaderStatusChange(reader.status);
    }
  }
}
