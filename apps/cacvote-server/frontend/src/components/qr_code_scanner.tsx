import jsqr from 'jsqr';
import { useEffect, useLayoutEffect, useRef } from 'react';
import { useVideoImage } from '../hooks/use_video_image';

interface Props {
  width?: string | number;
  height?: string | number;
  onCode: (code: string) => void;
  refreshInterval?: number;
}

export function QrCodeScanner({
  width,
  height,
  onCode,
  refreshInterval,
}: Props): JSX.Element {
  const videoRef = useRef<HTMLVideoElement>(null);
  const imageData = useVideoImage({ videoRef, refreshInterval });

  useLayoutEffect(() => {
    const video = videoRef.current;

    if (!video) {
      return;
    }

    let canceled = false;
    void (async () => {
      const stream = await navigator.mediaDevices.getUserMedia({
        video: { facingMode: 'environment' },
      });

      if (canceled) {
        return;
      }

      video.srcObject = stream;
      await video.play();
    })();

    return () => {
      canceled = true;
      video.srcObject = null;
    };
  }, []);

  useEffect(() => {
    if (!imageData) {
      return;
    }

    const code = jsqr(imageData.data, imageData.width, imageData.height);
    if (code) {
      onCode(code.data);
    }
  }, [imageData, onCode]);

  return (
    // eslint-disable-next-line jsx-a11y/media-has-caption
    <video ref={videoRef} width={width} height={height} />
  );
}
