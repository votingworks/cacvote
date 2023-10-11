import { Buffer } from 'buffer';
import { CommonAccessCardDetails } from '@votingworks/auth';

export type AuthStatus =
  | { status: 'no_card' }
  | { status: 'has_card'; card: CommonAccessCardDetails; isAdmin: boolean };

export interface Auth {
  /**
   * Gets the current auth status.
   */
  getAuthStatus(): Promise<AuthStatus>;

  /**
   * Checks the PIN for the current user.
   */
  checkPin(pin: string): Promise<boolean>;

  /**
   * Signs a message with the current user's private key.
   */
  generateSignature(message: Buffer, options: { pin: string }): Promise<Buffer>;

  /**
   * Gets the certificate for the current user.
   */
  getCertificate(): Promise<Buffer>;

  /**
   * Log out the current user.
   */
  logOut(): Promise<void>;
}
