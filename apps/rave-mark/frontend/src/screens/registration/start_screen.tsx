import { extractErrorMessage } from '@votingworks/basics';
import { Button, H1, Main, P, Screen, Select } from '@votingworks/ui';
import { useState } from 'react';
import { ServerId } from '@votingworks/rave-mark-backend';
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
  const [addressLine1, setAddressLine1] = useState('');
  const [addressLine2, setAddressLine2] = useState('');
  const [city, setCity] = useState('');
  const [state, setState] = useState('');
  const [postalCode, setPostalCode] = useState('');
  const [stateId, setStateId] = useState('');
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
      addressLine1,
      addressLine2,
      city,
      state,
      postalCode,
      stateId,
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
        </InlineForm>
        <InlineForm>
          <TextInput
            label="Address Line 1"
            data-testid="address-line-1"
            value={addressLine1}
            onChange={(newValue) => {
              setAddressLine1(newValue);
            }}
          />
        </InlineForm>
        <InlineForm>
          <TextInput
            label="Address Line 2"
            data-testid="address-line-2"
            value={addressLine2}
            onChange={(newValue) => {
              setAddressLine2(newValue);
            }}
          />
        </InlineForm>
        <InlineForm>
          <TextInput
            label="City"
            data-testid="city"
            value={city}
            onChange={(newValue) => {
              setCity(newValue);
            }}
          />
          <TextInput
            label="State"
            data-testid="state"
            value={state}
            onChange={(newValue) => {
              setState(newValue);
            }}
          />
          <TextInput
            label="Postal Code"
            data-testid="postal-code"
            value={postalCode}
            onChange={(newValue) => {
              setPostalCode(newValue);
            }}
          />
        </InlineForm>
        <InlineForm>
          <TextInput
            label="State ID / Driver’s License ID"
            data-testid="state-id"
            value={stateId}
            onChange={(newValue) => {
              setStateId(newValue);
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
