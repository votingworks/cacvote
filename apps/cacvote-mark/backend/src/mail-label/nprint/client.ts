import { assert, extractErrorMessage, iter } from '@votingworks/basics';
import { pdfToImages } from '@votingworks/image-utils';
import { LogEventId, Logger } from '@votingworks/logging';
import * as bmp from 'bmp-ts';
import { Buffer } from 'node:buffer';
import { spawn } from 'node:child_process';
import { unlink, writeFile } from 'node:fs/promises';
import { tmpNameSync } from 'tmp';
import { ChildProcessClient } from './child_process_client';
import { Event, IncomingMessage, Request, Response } from './types';

/**
 * The scale at which to render the mail label PDF as an image. This value was
 * experimentally determined.
 */
const MAIL_LABEL_NPRINT_RENDER_SCALE = 2.8;

/**
 * A client for the `libNPrint` binary wrapper used to print mail labels.
 */
export class Client {
  private nextRequestId = 0;

  /**
   * Connect to the `libNPrint` binary wrapper.
   *
   * @param bin The path to the `libNPrint` binary wrapper.
   * @param logger The logger to use for logging.
   * @param printer The name of the printer to use.
   * @returns A promise that resolves to a `Client` instance once connected.
   */
  static async connect(
    bin: string,
    logger: Logger,
    printer: string
  ): Promise<Client> {
    await logger.log(LogEventId.Info, 'system', {
      message: `Connecting to nprint printer ${printer} via ${bin}`,
    });

    const child = spawn(bin, { stdio: 'pipe' });

    child.once('close', (code, signal) => {
      void logger.log(LogEventId.Info, 'system', {
        message: `nprint client exited with code ${code} & signal ${signal}`,
      });
    });

    const client = new Client(new ChildProcessClient(child), logger);
    await client.sendRequest({ request: 'connect', printer });
    return client;
  }

  /**
   * Disconnect from the `libNPrint` binary wrapper.
   */
  async disconnect(): Promise<void> {
    if (!this.innerClient.isConnected()) {
      return;
    }

    await this.sendRequest({ request: 'disconnect' });
    await this.logger.log(LogEventId.Info, 'system', {
      message: 'nprint client disconnected',
    });
    await this.innerClient.close();
  }

  /**
   * Print a mail label from a PDF file.
   */
  async printPdf(pdfData: Buffer): Promise<void> {
    const rendered = await iter(
      pdfToImages(pdfData, { scale: MAIL_LABEL_NPRINT_RENDER_SCALE })
    ).first();
    assert(rendered, 'PDF failed to render; no images');

    // Encode the image as a BMP because `libNPrint` only supports BMP format.
    const bmpFilePath = tmpNameSync({ postfix: '.bmp' });
    await writeFile(
      bmpFilePath,
      bmp.encode(rendered.page as unknown as bmp.BmpImage).data
    );

    try {
      await this.logger.log(LogEventId.PrinterPrintRequest, 'system', {
        message: 'Printing label from bitmap image',
        labelPath: bmpFilePath,
      });
      await this.sendRequest({
        request: 'printLabel',
        labelPath: bmpFilePath,
      });
      await this.logger.log(LogEventId.PrinterPrintComplete, 'system', {
        message: 'Label printed successfully',
        labelPath: bmpFilePath,
        disposition: 'success',
      });
    } catch (error) {
      await this.logger.log(LogEventId.PrinterPrintComplete, 'system', {
        message: `Failed to print label: ${extractErrorMessage(error)}`,
        labelPath: bmpFilePath,
        disposition: 'error',
      });
    } finally {
      await unlink(bmpFilePath);
    }
  }

  private constructor(
    private readonly innerClient: ChildProcessClient<
      IncomingMessage,
      { type: 'response'; response: Response; inReplyTo: string },
      Event
    >,
    private readonly logger: Logger
  ) {}

  /**
   * Sends a request to the `libNPrint` binary wrapper and awaits the response.
   */
  private async sendRequest(request: Request): Promise<Response> {
    const id = this.nextRequestId;
    this.nextRequestId += 1;

    const { response } = await this.innerClient.send({
      request,
      replyTo: id.toString(),
    });

    return response;
  }
}
