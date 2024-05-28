import { H3, H5 } from '@votingworks/ui';
import React from 'react';
import styled from 'styled-components';

const WizardContainer = styled.div`
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  align-items: center;
  width: 100%;
  height: 100%;
`;

const WizardHeader = styled.div<{ centerContent?: boolean }>`
  text-align: center;
  margin-bottom: ${({ centerContent }) => (centerContent ? 'auto' : '0')};
`;

export function WizardHeaderTitle({
  step,
  title,
}: {
  step?: React.ReactNode;
  title: React.ReactNode;
}): JSX.Element {
  return (
    <div style={{ padding: '60px 20px' }}>
      <H5>{step}</H5>
      <H3>{title}</H3>
    </div>
  );
}

const WizardBody = styled.div`
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  align-items: center;
  width: 100%;
`;

const WizardFooter = styled.div<{ centerContent?: boolean }>`
  display: flex;
  flex-direction: row;
  justify-content: space-evenly;
  align-items: center;
  width: 100%;
  margin: auto auto 20px auto;
`;

const WizardActions = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: space-evenly;
  align-items: center;
  width: 100%;
  margin-top: 20px;
  padding: 20px;
  border-top: 3px solid #000000;
`;

const WizardActionsLeft = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: flex-start;
  align-items: center;
  width: 100%;
`;

const WizardActionsRight = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: flex-end;
  align-items: center;
  width: 100%;
`;

const ButtonBarContainer = styled.div<{ count: number }>`
  display: flex;
  flex-direction: row;
  justify-content: space-evenly;
  width: 100%;

  button {
    font-size: 0.7rem;
    min-width: ${({ count }) =>
      count === 1 ? '50%' : count === 2 ? '95%' : 'auto'};
  }
`;

export function WizardButtonBar({
  leftButton,
  rightButton,
}: {
  leftButton?: React.ReactNode;
  rightButton?: React.ReactNode;
}): JSX.Element {
  return (
    <ButtonBarContainer
      count={leftButton && rightButton ? 2 : leftButton || rightButton ? 1 : 0}
    >
      {leftButton && <WizardActionsLeft>{leftButton}</WizardActionsLeft>}
      {rightButton && <WizardActionsRight>{rightButton}</WizardActionsRight>}
    </ButtonBarContainer>
  );
}

export interface WizardProps {
  header: React.ReactNode;
  footer?: React.ReactNode;
  actions: React.ReactNode;
  children: React.ReactNode;
  centerContent?: boolean;
}

export function Wizard({
  header,
  footer,
  actions,
  children,
  centerContent,
}: WizardProps): JSX.Element {
  return (
    <WizardContainer>
      <WizardHeader centerContent={centerContent}>{header}</WizardHeader>
      <WizardBody>{children}</WizardBody>
      {footer && (
        <WizardFooter centerContent={centerContent}>{footer}</WizardFooter>
      )}
      <WizardActions>{actions}</WizardActions>
    </WizardContainer>
  );
}
