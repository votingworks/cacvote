import { Optional } from '@votingworks/basics';
import useInterval from 'use-interval';
import { RefObject, useMemo, useState } from 'react';

export function useVideoImage({
  videoRef,
  refreshInterval = 100,
}: {
  videoRef: RefObject<HTMLVideoElement>;
  refreshInterval?: number;
}): Optional<ImageData> {
  const [imageData, setImageData] = useState<ImageData>();
  const canvas = useMemo(() => document.createElement('canvas'), []);
  const context = useMemo(
    () => canvas.getContext('2d', { willReadFrequently: true }),
    [canvas]
  );

  useInterval(() => {
    const video = videoRef.current;
    if (!video || !context) {
      return;
    }

    if (video.videoWidth === 0 || video.videoHeight === 0) {
      return;
    }

    canvas.width = video.videoWidth;
    canvas.height = video.videoHeight;
    context.drawImage(video, 0, 0, canvas.width, canvas.height);
    setImageData(context.getImageData(0, 0, canvas.width, canvas.height));
  }, refreshInterval);

  return imageData;
}
