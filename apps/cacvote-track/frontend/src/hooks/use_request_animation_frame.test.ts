import { renderHook } from '@testing-library/react';
import { sleep } from '@votingworks/basics';
import { useRequestAnimationFrame } from './use_request_animation_frame';

test('does not call the callback if enabled=false', async () => {
  const callback = jest.fn();
  renderHook(() => useRequestAnimationFrame(callback, false));
  await sleep(100);
  expect(callback).not.toHaveBeenCalled();
});

test('calls the callback if enabled=true', async () => {
  const callback = jest.fn();
  renderHook(() => useRequestAnimationFrame(callback, true));
  await sleep(100);
  expect(callback).toHaveBeenCalled();
});

test('stops calling the callback if enabled=false', async () => {
  const callback = jest.fn();
  const { rerender } = renderHook(
    ({ enabled }) => useRequestAnimationFrame(callback, enabled),
    { initialProps: { enabled: true } }
  );
  await sleep(100);
  rerender({ enabled: false });
  callback.mockClear();
  await sleep(100);
  expect(callback).not.toHaveBeenCalled();
});
