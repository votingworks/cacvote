import { assert } from '@votingworks/basics';
import { ResponseApduError } from './apdu';

/**
 * An error that occurred while communicating with the smart card.
 */
export type SmartCardError =
  | {
      code: SmartCardErrorCode.NoCardReader;
      message: string;
    }
  | {
      code: SmartCardErrorCode.TransmitFailed;
      message: string;
    }
  | {
      code: SmartCardErrorCode.ResponseError;
      error: ResponseApduError;
    }
  | {
      code: SmartCardErrorCode.NoSpaceLeftOnDevice;
      message: string;
    }
  | {
      code: SmartCardErrorCode.AuthenticationError;
      message: string;
    }
  | {
      code: SmartCardErrorCode.AuthorizationError;
      message: string;
    }
  | {
      code: SmartCardErrorCode.UnknownError;
      message: string;
    };

/**
 * Possible error codes for smart card communication errors.
 */
export enum SmartCardErrorCode {
  /**
   * The smart card reader is not connected or not ready.
   */
  NoCardReader = 'no_card_reader',

  /**
   * Failure to transmit a command to the smart card.
   */
  TransmitFailed = 'transmit_failed',

  /**
   * Received an unexpected response from the smart card.
   */
  ResponseError = 'response_error',

  /**
   * No space left on device.
   */
  NoSpaceLeftOnDevice = 'no_space_left_on_device',

  /**
   * Authentication error.
   */
  AuthenticationError = 'authentication_error',

  /**
   * Authorization error.
   */
  AuthorizationError = 'authorization_error',

  /**
   * An unknown error occurred.
   */
  UnknownError = 'unknown_error',
}

/**
 * Create a {@link SmartCardError}.
 */
export function SmartCardError(
  code: SmartCardErrorCode.ResponseError,
  responseApduError: ResponseApduError
): SmartCardError;
/**
 * Create a {@link SmartCardError}.
 */
export function SmartCardError(
  code: Exclude<SmartCardErrorCode, SmartCardErrorCode.ResponseError>,
  message: string
): SmartCardError;
/**
 * Create a {@link SmartCardError}.
 */
export function SmartCardError(
  code: SmartCardErrorCode,
  messageOrError: string | ResponseApduError
): SmartCardError {
  if (typeof messageOrError === 'string') {
    assert(code !== SmartCardErrorCode.ResponseError);
    const error: SmartCardError = {
      code,
      message: messageOrError,
    };
    Object.defineProperty(error, 'toString', {
      enumerable: false,
      value: () => `${code}: ${messageOrError}`,
    });
    return error;
  }

  assert(code === SmartCardErrorCode.ResponseError);
  const error: SmartCardError = {
    code,
    error: messageOrError,
  };
  Object.defineProperty(error, 'toString', {
    enumerable: false,
    value: () => `${code}: ${messageOrError.message}`,
  });
  return error;
}
