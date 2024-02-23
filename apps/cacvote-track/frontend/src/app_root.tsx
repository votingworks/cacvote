import { throwIllegalValue } from '@votingworks/basics';
import { Button, H4, P } from '@votingworks/ui';
import jsQR from 'jsqr';
import React, { useEffect, useRef, useState } from 'react';
import { useRequestAnimationFrame } from './hooks/use_request_animation_frame';
import { useVideoDevice } from './hooks/use_video_device';

const EXPECTED_QR_CODE_DATA = "'Twas brillig";

type Stage = 'init' | 'scanning' | 'scan-success' | 'prompt-for-phone' | 'done';

const VIDEO_DEVICE_CONSTRAINTS: MediaStreamConstraints = {
  video: { facingMode: 'environment' },
};

export function AppRoot(): JSX.Element {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [stage, setStage] = useState<Stage>('init');
  const videoDevice = useVideoDevice({
    enabled: stage === 'scanning',
    constraints: VIDEO_DEVICE_CONSTRAINTS,
  });

  useRequestAnimationFrame(
    () => {
      const canvas = canvasRef.current;

      if (videoDevice && canvas) {
        const imageData = videoDevice.getCurrentFrame(canvas);
        const code = jsQR(imageData.data, imageData.width, imageData.height, {
          inversionAttempts: 'dontInvert',
        });

        if (code?.data === EXPECTED_QR_CODE_DATA) {
          setStage('scan-success');
        }
      }
    },
    typeof videoDevice !== 'undefined'
  );

  useEffect(() => {
    if (stage === 'scan-success') {
      const timeout = setTimeout(() => setStage('prompt-for-phone'), 1500);
      return () => clearTimeout(timeout);
    }
  }, [stage]);

  switch (stage) {
    case 'init':
      return (
        <React.Fragment>
          <H4 align="center">Track Your Ballot</H4>
          <P align="center">
            Scan your mailing label to see when your ballot is received and
            counted.
          </P>
          <Button variant="primary" onPress={() => setStage('scanning')}>
            Scan QR Code
          </Button>
        </React.Fragment>
      );

    case 'scanning':
      return (
        <React.Fragment>
          <H4>Scan QR Code</H4>
          <P>
            <canvas
              ref={canvasRef}
              style={{ maxWidth: '80vw', maxHeight: '35vh' }}
            />
          </P>
          <P align="center">
            Aim your camera at the QR code on your mailing label.
          </P>
          <P>
            <Button onPress={() => setStage('init')}>Cancel</Button>
          </P>
        </React.Fragment>
      );

    case 'scan-success':
      return <H4 align="center">✅ Ballot Tracker Scanned</H4>;

    case 'prompt-for-phone':
      return (
        <React.Fragment>
          <H4>Get Text Updates</H4>
          <P align="center">
            Enter your phone number to receive text updates when your ballot is
            received and counted:
          </P>
          <P align="center">
            <input type="tel" />
          </P>
          <P align="center">
            <Button onPress={() => setStage('done')} variant="primary">
              Submit
            </Button>
          </P>
        </React.Fragment>
      );

    case 'done':
      return (
        <React.Fragment>
          <H4>Thank You</H4>
          <P align="center">
            You will receive a text message when your ballot is received and
            counted.
          </P>
          <P align="center">You may now close this window.</P>
        </React.Fragment>
      );

    default:
      throwIllegalValue(stage);
  }
}
