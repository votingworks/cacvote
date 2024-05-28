import { Buffer } from 'buffer';

/**
 * An unencrypted ballot, similar to a CVR.
 *
 * @see https://github.com/JohnLCaron/egk-ec/blob/main/src/main/kotlin/org/cryptobiotic/eg/election/PlaintextBallot.kt
 */
export interface PlaintextBallot {
  ballot_id: string; // a unique ballot ID created by the external system
  ballot_style: string; // matches a Manifest.BallotStyle
  contests: PlaintextBallotContest[];
  sn?: number; // must be > 0
  errors?: string; // error messages from processing, eg when invalid
}

/**
 * A contest on a plaintext ballot.
 *
 * @see https://github.com/JohnLCaron/egk-ec/blob/main/src/main/kotlin/org/cryptobiotic/eg/election/PlaintextBallot.kt
 */
export interface PlaintextBallotContest {
  contest_id: string; // matches ContestDescription.contestId
  sequence_order: number; // matches ContestDescription.sequenceOrder
  selections: Selection[];
  write_ins: string[];
}

/**
 * A selection on a plaintext ballot.
 *
 * @see https://github.com/JohnLCaron/egk-ec/blob/main/src/main/kotlin/org/cryptobiotic/eg/election/PlaintextBallot.kt
 */
export interface PlaintextBallotSelection {
  selection_id: string; // matches SelectionDescription.selectionId
  sequence_order: number; // matches SelectionDescription.sequenceOrder
  vote: number;
}

/**
 * The type of election.
 */
export enum ElectionType {
  Primary = 'primary',
  General = 'general',
}

/**
 * A geopolitical unit, such as a district.
 */
export interface GeopoliticalUnit {
  object_id: Uuid;
  name: string;
  type: GeopoliticalUnitType;
  contact_information?: ContactInformation[];
}

/**
 * The type of geopolitical unit.
 */
export enum GeopoliticalUnitType {
  District = 'district',
}

/**
 * A political party.
 */
export interface Party {
  object_id: Uuid;
  name: string;
  abbreviation: string;
  color?: string;
  logo_uri?: string;
}

/**
 * A contest in an election.
 */
export interface Contest {
  object_id: Uuid;
  sequence_order: number;
  electoral_district_id: Uuid;
  vote_variation: VoteVariation;
  number_elected: number;
  votes_allowed: number;
  name: string;
  ballot_selections: BallotSelection[];
  ballot_title?: string;
  ballot_subtitle?: string;
}

/**
 * A candidate in an election.
 */
export interface Candidate {
  object_id: Uuid;
  name: string;
  party_id?: Uuid;
  image_url?: string;
  is_write_in: boolean;
}

/**
 * A ballot selection, i.e. a valid mark by a voter.
 */
export interface BallotSelection {
  object_id: Uuid;
  sequence_order: number;
  candidate_id: Uuid;
}

/**
 * The type of vote variation.
 */
export enum VoteVariation {
  OneOfM = 'one_of_m',
  NofM = 'n_of_m',
}

/**
 * A ballot style.
 */
export interface BallotStyle {
  object_id: Uuid;
  geopolitical_unit_ids: Uuid[];
  party_ids: Uuid[];
  image_uri?: string;
}

/**
 * Contact information for an election administrator.
 */
export interface ContactInformation {
  name: string;
  address_line: string[];
  email?: string;
  phone?: string;
}

/**
 * The ElectionGuard representation of an election manifest.
 */
export interface Manifest {
  election_scope_id: string;
  spec_version: string;
  type: ElectionType;
  start_date: string;
  end_date: string;
  geopolitical_units: GeopoliticalUnit[];
  parties: Party[];
  candidates: Candidate[];
  contests: Contest[];
  ballot_styles: BallotStyle[];
  name: string[];
  contact_information: ContactInformation;
}

/**
 * An encrypted version of a `PlaintextBallot`.
 */
export interface EncryptedBallot {
  ballot_id: string;
  ballot_style_id: string;
  encrypting_device: string;
  timestamp: number;
  code_baux: Buffer;
  confirmation_code: Buffer;
  election_id: Buffer;
  contests: unknown[];
  state: BallotState;
  encrypted_sn?: unknown;
  is_preencrypt: boolean;
}

/**
 * The state of a ballot.
 */
export enum BallotState {
  Cast = 'CAST',
  Challenged = 'CHALLENGED',
  Unknown = 'UNKNOWN',
}
