import { Id } from '@votingworks/types';
import { DateTime } from 'luxon';

export interface VoterInfo {
  /**
   * Database ID for a voter record.
   */
  id: Id;

  /**
   * Common Access Card ID for a voter.
   */
  commonAccessCardId: Id;

  /**
   * Whether or not this voter can perform admin actions.
   */
  isAdmin: boolean;
}

export interface VoterRegistrationInfo {
  /**
   * Database ID for a voter registration record.
   */
  id: Id;

  /**
   * Database ID for the associated voter record.
   */
  voterId: Id;

  /**
   * The voter's given name, i.e. first name.
   */
  givenName: string;

  /**
   * The voter's family name, i.e. last name.
   */
  familyName: string;

  /**
   * Database ID for a the election record associated with this voter
   * registration.
   */
  electionId?: Id;

  /**
   * The date and time at which this voter voted.
   */
  votedAt?: DateTime;
}
