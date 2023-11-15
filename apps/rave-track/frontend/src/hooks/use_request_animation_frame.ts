import { useEffect } from 'react';

export function useRequestAnimationFrame(
  callback: () => void,
  enabled = true
): void {
  useEffect(() => {
    if (!enabled) {
      return;
    }

    let animationFrameId: number | undefined;

    function tick() {
      callback();
      animationFrameId = requestAnimationFrame(tick);
    }

    animationFrameId = requestAnimationFrame(tick);

    return () => {
      if (animationFrameId !== undefined) {
        cancelAnimationFrame(animationFrameId);
      }
    };
  }, [callback, enabled]);
}
