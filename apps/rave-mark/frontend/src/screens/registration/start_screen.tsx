import { useState } from 'react';
import { Button, H1, Main, P, Screen } from '@votingworks/ui';
import { InlineForm, TextInput } from '../../components/text_input';
import { createVoterRegistration } from '../../api';

export function StartScreen(): JSX.Element {
  const [givenName, setGivenName] = useState('');
  const [familyName, setFamilyName] = useState('');
  const [addressLine1, setAddressLine1] = useState('');
  const [addressLine2, setAddressLine2] = useState('');
  const [city, setCity] = useState('');
  const [state, setState] = useState('');
  const [postalCode, setPostalCode] = useState('');
  const [stateId, setStateId] = useState('');
  const createVoterRegistrationMutation = createVoterRegistration.useMutation();

  function onSubmit() {
    createVoterRegistrationMutation.mutate({
      givenName,
      familyName,
      addressLine1,
      addressLine2,
      city,
      state,
      postalCode,
      stateId,
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
          <Button onPress={onSubmit}>Submit</Button>
        </InlineForm>
      </Main>
    </Screen>
  );
}
