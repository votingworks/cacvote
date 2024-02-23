import { extractErrorMessage } from '@votingworks/basics';
import { Button, H1, Main, P, Screen, Select } from '@votingworks/ui';
import { useState } from 'react';
import { ServerId } from '@votingworks/cacvote-mark-backend';
import {
  createVoterRegistration,
  getAuthStatus,
  getJurisdictions,
} from '../../api';
import { PinPadModal } from '../../components/pin_pad_modal';
import { InlineForm, TextInput } from '../../components/text_input';
import { COMMON_ACCESS_CARD_PIN_LENGTH } from '../../globals';

export function StartScreen(): JSX.Element {
  const authStatusQuery = getAuthStatus.useQuery();
  const getJurisdictionsQuery = getJurisdictions.useQuery();
  const cardDetails =
    authStatusQuery.data?.status === 'has_card'
      ? authStatusQuery.data.card
      : undefined;
  const [jurisdictionId, setJurisdictionId] = useState<ServerId>();
  const [givenName, setGivenName] = useState(cardDetails?.givenName ?? '');
  const [familyName, setFamilyName] = useState(cardDetails?.familyName ?? '');
  const [isShowingPinModal, setIsShowingPinModal] = useState(false);
  const createVoterRegistrationMutation = createVoterRegistration.useMutation();
  const error = createVoterRegistrationMutation.data?.err();

  function onChangeJurisdictionId(event: React.ChangeEvent<HTMLSelectElement>) {
    setJurisdictionId(event.target.value as ServerId);
  }

  function onSubmitRegistrationForm() {
    setIsShowingPinModal(true);
  }

  function onEnterPin(pin: string, jxId: ServerId) {
    createVoterRegistrationMutation.mutate({
      jurisdictionId: jxId,
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
          <Select value={jurisdictionId} onChange={onChangeJurisdictionId}>
            <option disabled selected={!jurisdictionId}>
              Select your jurisdiction
            </option>
            {getJurisdictionsQuery.data?.map((jx) => (
              <option key={jx.id} value={jx.id}>
                {jx.name}
              </option>
            ))}
          </Select>
          <Button onPress={onSubmitRegistrationForm} disabled={!jurisdictionId}>
            Submit
          </Button>
        </InlineForm>
        {isShowingPinModal && jurisdictionId && (
          <PinPadModal
            pinLength={COMMON_ACCESS_CARD_PIN_LENGTH}
            primaryButtonLabel={
              createVoterRegistrationMutation.isLoading
                ? 'Checking…'
                : 'Register'
            }
            dismissButtonLabel="Go Back"
            onEnter={(pin) => onEnterPin(pin, jurisdictionId)}
            onDismiss={() => setIsShowingPinModal(false)}
            disabled={createVoterRegistrationMutation.isLoading}
            error={error ? extractErrorMessage(error) : undefined}
          />
        )}
      </Main>
    </Screen>
  );
}
