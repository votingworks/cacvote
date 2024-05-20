import '@testing-library/jest-dom';

Object.defineProperty(navigator, 'mediaDevices', {
  value: {
    getUserMedia: jest.fn(),
  },
});

HTMLMediaElement.prototype.play = jest.fn();
