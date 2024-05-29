import { H4, Main, P, Screen } from '@votingworks/ui';
import styled from 'styled-components';
import { useEffect } from 'react';
import { CircledCheckmarkIcon } from '../../components/circled_checkmark_icon';

const NarrowP = styled(P)`
  font-weight: 300;
  margin: 40px auto;
  max-width: 66%;
  text-align: center;
`;

export function FinishedScreenStatic(): JSX.Element {
  return (
    <Screen>
      <Main centerChild>
        <CircledCheckmarkIcon />
        <H4>Don&rsquo;t forget to mail your ballot.</H4>
        <NarrowP>
          Your ballot will also be electronically delivered to your local
          election official.
        </NarrowP>
      </Main>
    </Screen>
  );
}

export interface FinishedScreenProps {
  onDone: () => void;
}

export function FinishedScreen({ onDone }: FinishedScreenProps): JSX.Element {
  useEffect(() => {
    const timeout = setTimeout(() => {
      onDone();
    }, 5000);

    return () => {
      clearTimeout(timeout);
    };
  }, [onDone]);

  return <FinishedScreenStatic />;
}
