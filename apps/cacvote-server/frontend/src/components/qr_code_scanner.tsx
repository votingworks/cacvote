import { Optional } from '@votingworks/basics';
import jsqr, { QRCode } from 'jsqr';
import { useEffect, useLayoutEffect, useRef } from 'react';
import { useVideoImage } from '../hooks/use_video_image';

export type ErrorType = 'no-camera' | 'decode-error';

interface Props {
  width?: string | number;
  height?: string | number;
  onCode: (code: string) => void;
  onError?: (type: ErrorType, error: unknown) => void;
  refreshInterval?: number;
}

export function QrCodeScanner({
  width,
  height,
  onCode,
  onError,
  refreshInterval,
}: Props): JSX.Element {
  const videoRef = useRef<HTMLVideoElement>(null);
  const imageData = useVideoImage({ videoRef, refreshInterval });

  useLayoutEffect(() => {
    const video = videoRef.current;
    let stream: Optional<MediaStream>;

    if (!video) {
      return;
    }

    let canceled = false;
    void (async () => {
      stream = await navigator.mediaDevices.getUserMedia({
        video: { facingMode: 'environment' },
      });

      if (canceled) {
        return;
      }

      video.srcObject = stream;
      await video.play();
    })().catch((error) => {
      onError?.('no-camera', error);
    });

    return () => {
      if (stream) {
        for (const track of stream.getTracks()) {
          track.stop();
        }
        stream = undefined;
      }
      canceled = true;
      video.srcObject = null;
    };
  }, [onError]);

  useEffect(() => {
    if (!imageData) {
      return;
    }

    let code: QRCode | null = null;

    try {
      code = jsqr(imageData.data, imageData.width, imageData.height);
    } catch (error) {
      onError?.('decode-error', error);
    }

    if (code) {
      onCode(code.data);
    }
  }, [imageData, onCode, onError]);

  return (
    // eslint-disable-next-line jsx-a11y/media-has-caption
    <video ref={videoRef} width={width} height={height} />
  );
}
