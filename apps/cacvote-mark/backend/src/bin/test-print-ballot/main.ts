import { Result, err, ok } from '@votingworks/basics';
import {
  FujitsuThermalPrinter,
  FujitsuThermalPrinterInterface,
  MockFileFujitsuPrinter,
  getFujitsuThermalPrinter,
} from '@votingworks/fujitsu-thermal-printer';
import { LogSource, Logger } from '@votingworks/logging';
import {
  BooleanEnvironmentVariableName,
  isFeatureFlagEnabled,
} from '@votingworks/utils';
import { readFile } from 'fs/promises';

const useMockPrinterByDefault = isFeatureFlagEnabled(
  BooleanEnvironmentVariableName.USE_MOCK_PRINTER
);

function usage(out: NodeJS.WriteStream) {
  out.write('Usage: test-print-ballot <PDF> [--printer <type>]\n');
  out.write('\n');
  out.write('Options:\n');
  out.write(
    `  --printer <type>  Specify whether to use a real or mock printer (default: ${
      useMockPrinterByDefault ? 'mock' : 'real'
    })\n`
  );
  out.write('  --help            Display this help message\n');
  out.write('\n');
  out.write('Example:\n');
  out.write('  test-print-ballot --printer mock\n');
}

export async function main(
  argv: readonly string[]
): Promise<Result<number, Error>> {
  const logger = new Logger(LogSource.System, () => Promise.resolve('system'));
  let printer: FujitsuThermalPrinterInterface;
  let inputPath: string | undefined;

  for (let i = 2; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--help' || arg === '--h') {
      usage(process.stdout);
      return ok(0);
    }

    if (arg === '--printer') {
      const printerType = argv[i + 1];
      switch (printerType) {
        case 'real':
          printer = new FujitsuThermalPrinter(logger);
          break;
        case 'mock':
          printer = new MockFileFujitsuPrinter(logger);
          break;
        default:
          throw new Error(`Invalid printer type: ${printerType}`);
      }
      i += 1;
    } else if (arg?.startsWith('-')) {
      usage(process.stderr);
      return ok(1);
    } else if (arg) {
      inputPath = arg;
    }
  }

  if (!inputPath) {
    usage(process.stderr);
    return ok(1);
  }

  printer ??= getFujitsuThermalPrinter(logger);

  const status = await printer.getStatus();

  if (status.state !== 'idle') {
    return err(new Error(`Printer is not ready. Got state=${status.state}`));
  }

  const pdf = await readFile(inputPath);
  const printResult = await printer.print(pdf);

  if (printResult.isErr()) {
    return err(new Error(JSON.stringify(printResult.err())));
  }

  await printer.close();

  return ok(0);
}
