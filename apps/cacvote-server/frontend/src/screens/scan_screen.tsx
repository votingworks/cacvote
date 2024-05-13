import { H3, Main, P, Screen } from '@votingworks/ui';
import { Buffer } from 'buffer';
import { postScannedCode } from '../api';
import { QrCodeScanner } from '../components/qr_code_scanner';

interface Props {
  onPostSuccess(): void;
}

/**
 * Presents the user with a screen to scan a QR code.
 */
export function ScanScreen({ onPostSuccess }: Props): JSX.Element {
  const postScannedCodeMutation = postScannedCode.useMutation();

  async function onCode(code: string) {
    const data = Buffer.from(code, 'base64');
    await postScannedCodeMutation.mutateAsync(data);
    onPostSuccess();
  }

  return (
    <Screen>
      <Main centerChild>
        <H3>Scan Mailing Label</H3>
        {!postScannedCodeMutation.isLoading && (
          <QrCodeScanner width="100%" onCode={onCode} />
        )}
        {postScannedCodeMutation.isLoading && <P>Sending…</P>}
      </Main>
    </Screen>
  );
}
