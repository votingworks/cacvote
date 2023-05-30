import { Button, Icons } from '@votingworks/ui';
import styled from 'styled-components';

const LabelContainer = styled.span`
  align-items: center;
  display: flex;
  flex-wrap: nowrap;
  font-weight: ${(p) => p.theme.sizes.fontWeight.semiBold};
  gap: 0.5rem;
  text-align: left;
`;

export interface DisplaySettingsButtonProps {
  onPress(): void;
}

export function DisplaySettingsButton({
  onPress,
}: DisplaySettingsButtonProps): JSX.Element {
  return (
    <Button onPress={onPress}>
      <LabelContainer>
        <Icons.Display />
        <span>Color & Size</span>
      </LabelContainer>
    </Button>
  );
}
