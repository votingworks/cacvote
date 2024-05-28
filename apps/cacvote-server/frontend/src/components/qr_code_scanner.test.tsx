import { renderWithThemes } from '@votingworks/ui';
import jsqr from 'jsqr';
import { QrCodeScanner } from './qr_code_scanner';
import { useVideoImage } from '../hooks/use_video_image';

jest.mock('../hooks/use_video_image', () => ({
  useVideoImage: jest.fn(),
}));

jest.mock('jsqr', () => jest.fn());

const useVideoImageMock = useVideoImage as jest.Mock;
const jsqrMock = jsqr as jest.Mock;

test('scan QR code', () => {
  Object.defineProperties(HTMLVideoElement.prototype, {
    videoWidth: {
      get: () => 100,
    },

    videoHeight: {
      get: () => 100,
    },
  });

  useVideoImageMock.mockReturnValue(null);

  jest.useFakeTimers();
  const onCode = jest.fn();

  // initial render
  renderWithThemes(<QrCodeScanner onCode={onCode} refreshInterval={1} />);

  jest.advanceTimersByTime(1);
  expect(jsqr).not.toHaveBeenCalled();

  useVideoImageMock.mockReturnValue({
    data: new Uint8ClampedArray(100),
    width: 100,
    height: 100,
  });

  // set up a frame without a QR code
  jsqrMock.mockReturnValue(null);
  renderWithThemes(<QrCodeScanner onCode={onCode} refreshInterval={1} />);
  jest.advanceTimersByTime(1);

  expect(jsqr).toHaveBeenCalled();
  expect(onCode).not.toHaveBeenCalled();

  // set up a frame with a QR code
  jsqrMock.mockReturnValue({ data: '123' });
  renderWithThemes(<QrCodeScanner onCode={onCode} refreshInterval={1} />);
  jest.advanceTimersByTime(1);

  expect(jsqr).toHaveBeenCalled();
  expect(onCode).toHaveBeenCalledWith('123');
});
