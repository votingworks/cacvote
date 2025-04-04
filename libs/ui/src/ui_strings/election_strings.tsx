/* eslint-disable react/destructuring-assignment */

import {
  BallotStyleId,
  Candidate,
  ContestLike,
  County,
  District,
  Election,
  ElectionStringKey as Key,
  LanguageCode,
  Party,
  Precinct,
  YesNoOption,
} from '@votingworks/types';
import { format } from '@votingworks/utils';

import { UiString } from './ui_string';
import { Pre } from '../typography';
import { DateString } from './date_string';
import { InEnglish, LanguageOverride } from './language_override';

type ContestWithDescription = ContestLike & {
  description: string;
};

/**
 * Election-specific strings that need to be translated and/or spoken.
 */
/* istanbul ignore next - mostly presentational, tested via apps where relevant */
export const electionStrings = {
  [Key.BALLOT_LANGUAGE]: (languageCode: LanguageCode) => (
    <LanguageOverride languageCode={languageCode}>
      <UiString uiStringKey={Key.BALLOT_LANGUAGE}>
        {format.languageDisplayName({ languageCode })}
      </UiString>
    </LanguageOverride>
  ),

  [Key.BALLOT_STYLE_ID]: (id: BallotStyleId) => (
    <InEnglish>
      <UiString uiStringKey={Key.BALLOT_STYLE_ID} uiStringSubKey={id}>
        {id}
      </UiString>
    </InEnglish>
  ),

  [Key.CANDIDATE_NAME]: (candidate: Candidate) => (
    <InEnglish>
      <UiString uiStringKey={Key.CANDIDATE_NAME} uiStringSubKey={candidate.id}>
        {candidate.name}
      </UiString>
    </InEnglish>
  ),

  [Key.CONTEST_DESCRIPTION]: (contest: ContestWithDescription) => (
    <UiString
      as={Pre}
      uiStringKey={Key.CONTEST_DESCRIPTION}
      uiStringSubKey={contest.id}
    >
      {contest.description}
    </UiString>
  ),

  [Key.CONTEST_OPTION_LABEL]: (option: YesNoOption) => (
    <UiString uiStringKey={Key.CONTEST_OPTION_LABEL} uiStringSubKey={option.id}>
      {option.label}
    </UiString>
  ),

  [Key.CONTEST_TITLE]: (contest: ContestLike) => (
    <UiString uiStringKey={Key.CONTEST_TITLE} uiStringSubKey={contest.id}>
      {contest.title}
    </UiString>
  ),

  [Key.COUNTY_NAME]: (county: County) => (
    <UiString uiStringKey={Key.COUNTY_NAME} uiStringSubKey={county.id}>
      {county.name}
    </UiString>
  ),

  [Key.DISTRICT_NAME]: (district: District) => (
    <UiString uiStringKey={Key.DISTRICT_NAME} uiStringSubKey={district.id}>
      {district.name}
    </UiString>
  ),

  [Key.ELECTION_DATE]: (election: Election) => (
    <UiString uiStringKey={Key.ELECTION_DATE}>
      <DateString value={new Date(election.date)} />
    </UiString>
  ),

  [Key.ELECTION_TITLE]: (election: Election) => (
    <UiString uiStringKey={Key.ELECTION_TITLE}>{election.title}</UiString>
  ),

  [Key.PARTY_FULL_NAME]: (party: Party) => (
    <UiString uiStringKey={Key.PARTY_NAME} uiStringSubKey={party.id}>
      {party.fullName}
    </UiString>
  ),

  [Key.PARTY_NAME]: (party: Party) => (
    <UiString uiStringKey={Key.PARTY_NAME} uiStringSubKey={party.id}>
      {party.name}
    </UiString>
  ),

  [Key.PRECINCT_NAME]: (precinct: Precinct) => (
    <UiString uiStringKey={Key.PRECINCT_NAME} uiStringSubKey={precinct.id}>
      {precinct.name}
    </UiString>
  ),

  [Key.STATE_NAME]: (election: Election) => (
    <UiString uiStringKey={Key.STATE_NAME}>{election.state}</UiString>
  ),
} as const;
// TODO(kofi): Update esbuild so we can use the `satisfies` operator here:
// } satisfies Record<Key, unknown>;
