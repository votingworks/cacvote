import { Result, ok } from '@votingworks/basics';
import { LogSource, Logger } from '@votingworks/logging';
import { Buffer } from 'buffer';
import * as mailLabel from '../../mail-label';
import { resolveWorkspace } from '../../workspace';

export async function main(
  argv: readonly string[]
): Promise<Result<void, Error>> {
  const logger = new Logger(LogSource.System, () => Promise.resolve('system'));
  const workspace = await resolveWorkspace(logger);
  let printer: mailLabel.printing.LabelPrinterInterface;

  for (let i = 2; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--printer') {
      const printerType = argv[i + 1];
      switch (printerType) {
        case 'real':
          printer = await mailLabel.printing.getRealLabelPrinter(logger);
          break;
        case 'mock':
          printer = mailLabel.printing.getMockLabelPrinter(workspace, logger);
          break;
        default:
          throw new Error(`Invalid printer type: ${printerType}`);
      }
      i += 1;
    }
  }

  printer ??= await mailLabel.printing.getLabelPrinter(workspace, logger);

  const pdf = await mailLabel.rendering.buildPdf({
    mailingAddress: '123 Main St\nAnytown, CA 90210',
    qrCodeData: Buffer.from(
      'BkUCAzAwMAMKMDEyMzQ1Njc4OQQQNysj0vgURf+Gi2Xo1o6dAgUgEIxqEkEQ0QTGUEQzLmsp8NAMBUw3opShGVfJ2H3WHDIHRzBFAiEAubQrU91NQ/HejiIGWYPkX29QWDZ75ofJO7jBqdwSq7kCIATMY7NHAOuV3Dm9lbLUD8yF2VpiyOomQGmEyIitm0eE'
    ),
  });

  await printer.printPdf(pdf);
  await printer.close();

  return ok();
}
