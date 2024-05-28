import { useId } from 'react';
import styled from 'styled-components';

const RadioBoxContainer = styled.div`
  position: relative;
`;

const RadioBoxInput = styled.input`
  display: block;
  position: absolute;
  visibility: hidden;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
`;

const RadioBoxStyled = styled.label<{ selected?: boolean }>`
  display: flex;
  align-items: center;
  justify-content: center;
  width: 340px;
  height: 340px;
  padding: 20px;

  background: ${({ selected }) => (selected ? '#4a44a5' : '#ffffff')};
  border: 2px solid ${({ selected }) => (selected ? '#4a44a5' : '#000000')};
  border-radius: 8px;

  font-style: normal;
  font-weight: 400;
  font-size: 35px;
  line-height: 50px;
  text-align: center;

  color: ${({ selected }) => (selected ? '#ffffff' : '#000000')};
  pointer-events: none;
`;

export function RadioBox({
  selected,
  onClick,
  children,
}: {
  selected: boolean;
  onClick: () => void;
  children: React.ReactNode;
}): JSX.Element {
  const id = useId();

  return (
    <RadioBoxContainer onClick={onClick}>
      <RadioBoxInput type="radio" checked={selected} readOnly id={id} />
      <RadioBoxStyled htmlFor={id} selected={selected}>
        <div>{children}</div>
      </RadioBoxStyled>
    </RadioBoxContainer>
  );
}
