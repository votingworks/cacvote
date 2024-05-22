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

const WizardHeader = styled.div`
  text-align: center;
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

const WizardFooter = styled.div`
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

export function WizardButtonBar({
  leftButton,
  rightButton,
}: {
  leftButton?: React.ReactNode;
  rightButton?: React.ReactNode;
}): JSX.Element {
  return (
    <React.Fragment>
      {leftButton && <WizardActionsLeft>{leftButton}</WizardActionsLeft>}
      {rightButton && <WizardActionsRight>{rightButton}</WizardActionsRight>}
    </React.Fragment>
  );
}

export interface WizardProps {
  header: React.ReactNode;
  footer?: React.ReactNode;
  actions: React.ReactNode;
  children: React.ReactNode;
}

export function Wizard({
  header,
  footer,
  actions,
  children,
}: WizardProps): JSX.Element {
  return (
    <WizardContainer>
      <WizardHeader>{header}</WizardHeader>
      <WizardBody>{children}</WizardBody>
      {footer && <WizardFooter>{footer}</WizardFooter>}
      <WizardActions>{actions}</WizardActions>
    </WizardContainer>
  );
}
