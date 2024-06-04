import { Button, H3, Icons, Main, P, Screen, Text } from '@votingworks/ui';
import { Buffer } from 'buffer';
import React, { useState } from 'react';
import { postScannedCode } from '../api';
import { ErrorType, QrCodeScanner } from '../components/qr_code_scanner';

/**
 * Presents the user with a screen to scan a QR code.
 */
export function ScanScreen(): JSX.Element {
  const [scanError, setScanError] = useState<[ErrorType, unknown]>();
  const postScannedCodeMutation = postScannedCode.useMutation();

  function onCode(code: string) {
    if (!postScannedCodeMutation.isIdle) {
      return;
    }

    const data = Buffer.from(code, 'base64');
    postScannedCodeMutation.mutate(data);
  }

  function reset() {
    setScanError(undefined);
    postScannedCodeMutation.reset();
  }

  return (
    <Screen>
      <Main centerChild padded>
        {postScannedCodeMutation.data?.isOk() ? (
          <React.Fragment>
            <Text success>
              <Icons.Done /> Mail label scan success!
            </Text>
            <Button onPress={reset}>Scan again</Button>
          </React.Fragment>
        ) : postScannedCodeMutation.data?.isErr() ? (
          <React.Fragment>
            <Text error>
              <Icons.Warning /> Could not scan mail label:
              <br />
              {postScannedCodeMutation.data?.err()}
            </Text>
            <Button onPress={reset}>Try again</Button>
          </React.Fragment>
        ) : (
          <React.Fragment>
            <H3>Scan Mail Label</H3>
            {!postScannedCodeMutation.isLoading && !scanError && (
              <QrCodeScanner
                width="100%"
                onCode={onCode}
                onError={(type, newError) => setScanError([type, newError])}
              />
            )}
            {postScannedCodeMutation.isLoading && <P>Sending…</P>}
            {scanError?.[0] === 'no-camera' && (
              <React.Fragment>
                <Text error>
                  <Icons.Warning /> Failed to scan mail label. Does this device
                  have a camera?
                  <br />
                  {JSON.stringify(scanError[1])}
                </Text>
                <Button onPress={reset}>Try again</Button>
              </React.Fragment>
            )}
            {scanError?.[0] === 'decode-error' && (
              <React.Fragment>
                <Text error>
                  <Icons.Warning /> Failed to scan mail label. Is the QR code
                  damaged or obscured?
                </Text>
                <Button onPress={reset}>Try again</Button>
              </React.Fragment>
            )}
          </React.Fragment>
        )}
      </Main>
    </Screen>
  );
}
