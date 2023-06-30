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
            placeholder="Given Name"
            data-testid="given-name"
            value={givenName}
            onInput={(event) => {
              setGivenName(event.currentTarget.value);
            }}
          />
          <TextInput
            placeholder="Family Name"
            data-testid="family-name"
            value={familyName}
            onInput={(event) => {
              setFamilyName(event.currentTarget.value);
            }}
          />
        </InlineForm>
        <InlineForm>
          <TextInput
            placeholder="Address Line 1"
            data-testid="address-line-1"
            value={addressLine1}
            onInput={(event) => {
              setAddressLine1(event.currentTarget.value);
            }}
          />
        </InlineForm>
        <InlineForm>
          <TextInput
            placeholder="Address Line 2"
            data-testid="address-line-2"
            value={addressLine2}
            onInput={(event) => {
              setAddressLine2(event.currentTarget.value);
            }}
          />
        </InlineForm>
        <InlineForm>
          <TextInput
            placeholder="City"
            data-testid="city"
            value={city}
            onInput={(event) => {
              setCity(event.currentTarget.value);
            }}
          />
          <TextInput
            placeholder="State"
            data-testid="state"
            value={state}
            onInput={(event) => {
              setState(event.currentTarget.value);
            }}
          />
          <TextInput
            placeholder="Postal Code"
            data-testid="postal-code"
            value={postalCode}
            onInput={(event) => {
              setPostalCode(event.currentTarget.value);
            }}
          />
        </InlineForm>
        <InlineForm>
          <TextInput
            placeholder="State ID / Driver’s License ID"
            data-testid="state-id"
            value={stateId}
            onInput={(event) => {
              setStateId(event.currentTarget.value);
            }}
          />
          <Button onPress={onSubmit}>Submit</Button>
        </InlineForm>
      </Main>
    </Screen>
  );
}
