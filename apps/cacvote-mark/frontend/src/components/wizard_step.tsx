import { throwIllegalValue } from '@votingworks/basics';
import styled from 'styled-components';

const CIRCLE_RADIUS = 20;

const Container = styled.div`
  font-size: 20px;

  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;

  > *:first-child {
    margin-bottom: 10px;
  }
`;

const Circle = styled.div<{ color?: string; stroke?: string; fill?: string }>`
  font-weight: 500;
  width: ${CIRCLE_RADIUS * 2}px;
  height: ${CIRCLE_RADIUS * 2}px;
  border-radius: 50%;
  background-color: ${({ fill = 'none' }) => fill};
  border: ${({ stroke }) => (stroke ? `3px solid ${stroke}` : 'none')};
  display: flex;
  justify-content: center;
  align-items: center;
  color: ${({ color = 'inherit' }) => color};

  :after {
    content: counter(step);
    counter-increment: step;
  }
`;

const Title = styled.div<{ bold?: boolean }>`
  text-align: center;
  font-weight: ${({ bold }) => (bold ? 'bold' : 'normal')};
`;

export type StepState = 'active' | 'done' | 'todo';

/**
 * A step in a wizard.
 */
export function WizardStep({
  title,
  state,
}: {
  title: string;
  state: StepState;
}): JSX.Element {
  switch (state) {
    case 'active':
      return (
        <Container data-state={state}>
          <Circle stroke="#A67ADF" />
          <Title bold>{title}</Title>
        </Container>
      );
    case 'done':
      return (
        <Container data-state={state}>
          <Circle color="white" fill="#A67ADF" />
          <Title>{title}</Title>
        </Container>
      );
    case 'todo':
      return (
        <Container data-state={state}>
          <Circle color="white" fill="#646464" />
          <Title>{title}</Title>
        </Container>
      );
    default:
      throwIllegalValue(state);
  }
}

const CIRCLE_LINE_PADDING = 6;
const LINE_STROKE_WIDTH = 4;

const WizardStepsContainer = styled.div`
  display: flex;
  flex-direction: row;
  width: 100%;

  counter-reset: step;

  > * {
    position: relative;
    flex: 1;
  }

  > *:not(:first-child):before {
    content: '';
    width: calc(100% - ${(CIRCLE_RADIUS + CIRCLE_LINE_PADDING) * 2}px);
    height: ${LINE_STROKE_WIDTH}px;
    background-color: #ddd;
    position: absolute;
    border-radius: ${LINE_STROKE_WIDTH / 2}px;
    left: calc(-50% + ${CIRCLE_RADIUS + CIRCLE_LINE_PADDING}px);
    top: calc(${CIRCLE_RADIUS}px - ${LINE_STROKE_WIDTH / 2}px);
  }

  > *:not([data-state='todo']):not(:first-child):before {
    background-color: #a67adf;
  }
`;

export function WizardSteps<T extends string>({
  current,
  steps,
}: {
  current: NoInfer<T>;
  steps: Array<{ id: T; title: string }>;
}): JSX.Element {
  const currentIndex = steps.findIndex((step) => step.id === current);

  return (
    <WizardStepsContainer>
      {steps.map((step, index) => (
        <WizardStep
          key={step.id}
          state={
            index < currentIndex
              ? 'done'
              : index === currentIndex
              ? 'active'
              : 'todo'
          }
          title={step.title}
        />
      ))}
    </WizardStepsContainer>
  );
}
