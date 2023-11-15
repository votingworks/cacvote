import { useCallback, useEffect, useMemo, useState } from 'react';

export interface UseVideoDeviceProps {
  enabled?: boolean;
  constraints: MediaStreamConstraints;
}

export interface VideoDevice {
  camera: MediaStream;
  width: number;
  height: number;
  getCurrentFrame(canvas: HTMLCanvasElement): ImageData;
}

export function useVideoDevice({
  enabled,
  constraints,
}: UseVideoDeviceProps): VideoDevice | undefined {
  const video = useMemo(() => document.createElement('video'), []);
  const [camera, setCamera] = useState<MediaStream>();
  const [width, setWidth] = useState<number>();
  const [height, setHeight] = useState<number>();

  const updateSize = useCallback(() => {
    setWidth(video.videoWidth);
    setHeight(video.videoHeight);
  }, [video]);

  const setup = useCallback(async () => {
    const stream = await navigator.mediaDevices.getUserMedia(constraints);
    video.srcObject = stream;
    // tell iOS Safari we don't want fullscreen
    video.setAttribute('playsinline', 'true');
    await video.play();
    video.addEventListener('loadedmetadata', updateSize);
    setCamera(stream);
  }, [constraints, updateSize, video]);

  const teardown = useCallback(() => {
    const stream = video.srcObject as MediaStream | undefined;
    for (const track of stream?.getTracks() ?? []) {
      track.stop();
    }
    video.srcObject = null;
    video.removeEventListener('loadedmetadata', updateSize);
    setCamera(undefined);
    setWidth(undefined);
    setHeight(undefined);
  }, [updateSize, video]);

  useEffect(() => {
    if (enabled) {
      void setup();
    } else {
      teardown();
    }

    return teardown;
  }, [enabled, setup, teardown]);

  return useMemo(
    () =>
      !enabled ||
      typeof camera === 'undefined' ||
      typeof width === 'undefined' ||
      typeof height === 'undefined' ||
      video.readyState !== video.HAVE_ENOUGH_DATA
        ? undefined
        : {
            camera,
            width,
            height,
            getCurrentFrame(canvas: HTMLCanvasElement) {
              /* eslint-disable no-param-reassign */
              canvas.width = width;
              canvas.height = height;
              /* eslint-enable no-param-reassign */
              const context = canvas.getContext('2d');
              if (context) {
                context.drawImage(video, 0, 0, width, height);
                return context.getImageData(0, 0, width, height);
              }
              throw new Error('Could not get canvas context');
            },
          },
    [enabled, camera, width, height, video]
  );
}
