import { LogEventId, Logger } from '@votingworks/logging';
import {
  BooleanEnvironmentVariableName,
  isFeatureFlagEnabled,
} from '@votingworks/utils';
import { Buffer } from 'node:buffer';
import { mkdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { LIBNPRINT_WRAPPER_PATH } from '../globals';
import { Workspace } from '../workspace';
import { Client } from './nprint/client';

/**
 * This is the name `libNPrint` gives the default printer. In the future we may
 * want to allow users to list available printers.
 */
export const DEFAULT_PRINTER_NAME = 'PRT001';

export interface LabelPrinterInterface {
  printPdf(pdfData: Buffer): Promise<void>;
  close(): Promise<void>;
}

export class MockLabelPrinter implements LabelPrinterInterface {
  constructor(
    private readonly workspace: Workspace,
    private readonly logger: Logger
  ) {}

  async printPdf(pdfData: Buffer): Promise<void> {
    const dirname = join(this.workspace.path, 'prints');
    const filename = join(dirname, `print-job-${new Date().toISOString()}.pdf`);
    await this.logger.logAsCurrentRole(LogEventId.PrinterPrintRequest, {
      message: 'Printing with mock label printer to workspace path',
      filename,
    });

    await mkdir(dirname, { recursive: true });
    await writeFile(filename, pdfData);

    await this.logger.logAsCurrentRole(LogEventId.PrinterPrintComplete, {
      message: 'Printed with mock label printer to workspace path',
      filename,
      disposition: 'success',
    });
  }

  async close(): Promise<void> {
    await this.logger.logAsCurrentRole(LogEventId.Info, {
      message: 'Closing mock label printer',
    });
  }
}

export async function getRealLabelPrinter(
  logger: Logger
): Promise<LabelPrinterInterface> {
  const client = await Client.connect(
    LIBNPRINT_WRAPPER_PATH,
    logger,
    DEFAULT_PRINTER_NAME
  );
  return {
    printPdf: async (pdfData: Buffer) => {
      await client.printPdf(pdfData);
    },

    close: async () => {
      await client.disconnect();
    },
  };
}

export function getMockLabelPrinter(
  workspace: Workspace,
  logger: Logger
): LabelPrinterInterface {
  return new MockLabelPrinter(workspace, logger);
}

export async function getLabelPrinter(
  workspace: Workspace,
  logger: Logger
): Promise<LabelPrinterInterface> {
  if (isFeatureFlagEnabled(BooleanEnvironmentVariableName.USE_MOCK_PRINTER)) {
    return getMockLabelPrinter(workspace, logger);
  }

  return await getRealLabelPrinter(logger);
}
