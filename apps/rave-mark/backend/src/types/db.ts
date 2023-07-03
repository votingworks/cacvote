import { BallotStyleId, Id, PrecinctId, VotesDict } from '@votingworks/types';
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

export interface VoterRegistrationRequest {
  /**
   * Database ID for a voter registration request record.
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
   * Voter's address line 1.
   */
  addressLine1: string;

  /**
   * Voter's address line 2.
   */
  addressLine2?: string;

  /**
   * Voter's city.
   */
  city: string;

  /**
   * Voter's state.
   */
  state: string;

  /**
   * Voter's postal code.
   */
  postalCode: string;

  /**
   * Voter's state ID.
   */
  stateId: string;
}

export interface VoterElectionRegistration {
  /**
   * Database ID for an election registration record.
   */
  id: Id;

  /**
   * Database ID for a voter record.
   */
  voterId: Id;

  /**
   * Database ID for the voter's registration request record.
   */
  voterRegistrationRequestId: Id;

  /**
   * Database ID for a the election record associated with this voter
   * registration.
   */
  electionId: Id;

  /**
   * Precinct ID for the voter's precinct.
   */
  precinctId: PrecinctId;

  /**
   * Ballot style ID for the voter's ballot style.
   */
  ballotStyleId: BallotStyleId;

  /**
   * Date and time when the voter registered for the election.
   */
  createdAt: DateTime;
}

export interface VoterElectionSelection {
  /**
   * Database ID for a voter record.
   */
  voterId: Id;

  /**
   * Database ID for the voter's election registration record.
   */
  voterElectionRegistrationId: Id;

  /**
   * Votes cast by the voter.
   */
  votes: VotesDict;

  /**
   * Date and time when the voter cast their votes.
   */
  createdAt: DateTime;
}
