import { extractErrorMessage } from '@votingworks/basics';
import { Button, H1, Main, P, Screen, Select } from '@votingworks/ui';
import { useState } from 'react';
import { JurisdictionCode } from '@votingworks/cacvote-mark-backend';
import {
  createVoterRegistration,
  getAuthStatus,
  getJurisdictionCodes,
} from '../../api';
import { PinPadModal } from '../../components/pin_pad_modal';
import { InlineForm, TextInput } from '../../components/text_input';
import { COMMON_ACCESS_CARD_PIN_LENGTH } from '../../globals';

export function StartScreen(): JSX.Element {
  const authStatusQuery = getAuthStatus.useQuery();
  const getJurisdictionsQuery = getJurisdictionCodes.useQuery();
  const cardDetails =
    authStatusQuery.data?.status === 'has_card'
      ? authStatusQuery.data.card
      : undefined;
  const [jurisdictionCode, setJurisdictionCode] = useState<JurisdictionCode>();
  const [givenName, setGivenName] = useState(cardDetails?.givenName ?? '');
  const [familyName, setFamilyName] = useState(cardDetails?.familyName ?? '');
  const [isShowingPinModal, setIsShowingPinModal] = useState(false);
  const createVoterRegistrationMutation = createVoterRegistration.useMutation();
  const error = createVoterRegistrationMutation.data?.err();

  function onChangeJurisdictionId(event: React.ChangeEvent<HTMLSelectElement>) {
    setJurisdictionCode(event.target.value as JurisdictionCode);
  }

  function onSubmitRegistrationForm() {
    setIsShowingPinModal(true);
  }

  function onEnterPin(pin: string, code: JurisdictionCode) {
    createVoterRegistrationMutation.mutate({
      jurisdictionCode: code,
      givenName,
      familyName,
      pin,
    });
  }

  return (
    <Screen>
      <Main>
        <H1>Registration</H1>
        <P>Fill out your information to register to vote.</P>
        <InlineForm>
          <TextInput
            label="Given Name"
            data-testid="given-name"
            value={givenName}
            onChange={(newValue) => {
              setGivenName(newValue);
            }}
          />
          <TextInput
            label="Family Name"
            data-testid="family-name"
            value={familyName}
            onChange={(newValue) => {
              setFamilyName(newValue);
            }}
          />
          <Select value={jurisdictionCode} onChange={onChangeJurisdictionId}>
            <option disabled selected={!jurisdictionCode}>
              Select your jurisdiction
            </option>
            {getJurisdictionsQuery.data?.map((jx) => (
              <option key={jx} value={jx}>
                {jx}
              </option>
            ))}
          </Select>
          <Button
            onPress={onSubmitRegistrationForm}
            disabled={!jurisdictionCode}
          >
            Submit
          </Button>
        </InlineForm>
        {isShowingPinModal && jurisdictionCode && (
          <PinPadModal
            pinLength={COMMON_ACCESS_CARD_PIN_LENGTH}
            primaryButtonLabel={
              createVoterRegistrationMutation.isLoading
                ? 'Checking…'
                : 'Register'
            }
            dismissButtonLabel="Go Back"
            onEnter={(pin) => onEnterPin(pin, jurisdictionCode)}
            onDismiss={() => setIsShowingPinModal(false)}
            disabled={createVoterRegistrationMutation.isLoading}
            error={error ? extractErrorMessage(error) : undefined}
          />
        )}
      </Main>
    </Screen>
  );
}
