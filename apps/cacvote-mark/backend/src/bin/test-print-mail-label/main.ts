import { Result, ok, throwIllegalValue } from '@votingworks/basics';
import { LogSource, Logger } from '@votingworks/logging';
import {
  BooleanEnvironmentVariableName,
  isFeatureFlagEnabled,
} from '@votingworks/utils';
import { Buffer } from 'buffer';
import { readFile } from 'fs/promises';
import * as mailLabel from '../../mail-label';
import { Store } from '../../store';
import { resolveWorkspace } from '../../workspace';

const useMockPrinterByDefault = isFeatureFlagEnabled(
  BooleanEnvironmentVariableName.USE_MOCK_PRINTER
);

function usage(out: NodeJS.WriteStream) {
  out.write('Usage: test-print-mail-label [--printer <type>]\n');
  out.write('\n');
  out.write('Options:\n');
  out.write(' --input <path>     Specify the path to the input file\n');
  out.write(
    `  --printer <type>  Specify whether to use a real or mock printer (default: ${
      useMockPrinterByDefault ? 'mock' : 'real'
    })\n`
  );
  out.write('  --help            Display this help message\n');
  out.write('\n');
  out.write('Example:\n');
  out.write('  test-print-mail-label --printer mock\n');
}

export async function main(
  argv: readonly string[]
): Promise<Result<number, Error>> {
  const logger = new Logger(LogSource.System, () =>
    Promise.resolve('system' as const)
  );
  const workspace = await resolveWorkspace(logger, Store);
  let printerType: 'real' | 'mock' = 'real';
  let inputPath: string | undefined;

  for (let i = 2; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--help' || arg === '--h') {
      usage(process.stdout);
      return ok(0);
    }

    if (arg === '--printer') {
      const cliPrinterType = argv[i + 1];
      switch (cliPrinterType) {
        case 'real':
        case 'mock':
          printerType = cliPrinterType;
          break;
        default:
          throw new Error(`Invalid printer type: ${printerType}`);
      }
      i += 1;
    } else if (arg === '--input') {
      inputPath = argv[i + 1];
      i += 1;
    } else if (arg?.startsWith('-')) {
      usage(process.stderr);
      return ok(1);
    }
  }

  const pdf = inputPath
    ? await readFile(inputPath)
    : await mailLabel.rendering.buildPdf({
        mailingAddress: '123 Main St\nAnytown, CA 90210',
        qrCodeData: Buffer.from(
          'BkUCAzAwMAMKMDEyMzQ1Njc4OQQQNysj0vgURf+Gi2Xo1o6dAgUgEIxqEkEQ0QTGUEQzLmsp8NAMBUw3opShGVfJ2H3WHDIHRzBFAiEAubQrU91NQ/HejiIGWYPkX29QWDZ75ofJO7jBqdwSq7kCIATMY7NHAOuV3Dm9lbLUD8yF2VpiyOomQGmEyIitm0eE'
        ),
      });

  let printer: mailLabel.printing.LabelPrinterInterface | undefined;
  try {
    printer =
      printerType === 'mock'
        ? mailLabel.printing.getMockLabelPrinter(workspace, logger)
        : printerType === 'real'
        ? await mailLabel.printing.getLabelPrinter(workspace, logger)
        : throwIllegalValue(printerType);
    await printer.printPdf(pdf);
  } finally {
    await printer?.close();
  }

  return ok(0);
}
