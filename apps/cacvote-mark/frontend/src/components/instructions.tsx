import { P } from '@votingworks/ui';
import React from 'react';
import styled from 'styled-components';

export const InstructionSection = styled.div<{ padded?: boolean }>`
  display: flex;
  border: 3px solid #000000;
  border-radius: 2px;
  width: 90%;
  margin-top: 30px;
  padding: ${({ padded = true }) => (padded ? '40px 50px 40px 50px' : '0')};
`;

export const Instructions = styled(P)`
  flex: 1;
  display: flex;
  align-items: center;
  padding-left: 50px;
  font-weight: 400;
`;

const IconContainer = styled.div`
  margin: auto 0 0 5%;
`;

export function IconWithInstructions({
  icon,
  children,
}: {
  icon: React.ReactNode;
  children: React.ReactNode;
}): JSX.Element {
  return (
    <React.Fragment>
      <IconContainer>{icon}</IconContainer>
      <Instructions>{children}</Instructions>
    </React.Fragment>
  );
}
