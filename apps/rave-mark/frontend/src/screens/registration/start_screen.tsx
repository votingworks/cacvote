import { useState } from 'react';
import { Button, H1, Main, P, Screen } from '@votingworks/ui';
import { InlineForm, TextInput } from '../../components/text_input';
import { createVoterRegistration } from '../../api';

export function StartScreen(): JSX.Element {
  const [givenName, setGivenName] = useState('');
  const [familyName, setFamilyName] = useState('');
  const createVoterRegistrationMutation = createVoterRegistration.useMutation();

  function onSubmit() {
    createVoterRegistrationMutation.mutate({
      givenName,
      familyName,
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
          <Button onPress={onSubmit}>Submit</Button>
        </InlineForm>
      </Main>
    </Screen>
  );
}
