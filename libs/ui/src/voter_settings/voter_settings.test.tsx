import userEvent from '@testing-library/user-event';
import { ThemeConsumer } from 'styled-components';
import { UiTheme } from '@votingworks/types';
import { render, screen } from '../../test/react_testing_library';
import { VoterSettings } from '.';

test('renders expected subcomponents', () => {
  render(<VoterSettings onClose={jest.fn()} />);

  screen.getByRole('heading', { name: 'Settings:' });
  screen.getByRole('tablist', { name: 'Settings' });
  screen.getByRole('tabpanel');
  screen.getByRole('radiogroup', { name: 'Color Contrast Settings' });
});

test('changes tab pane on tab bar events', () => {
  render(<VoterSettings onClose={jest.fn()} />);

  screen.getByRole('radiogroup', { name: 'Color Contrast Settings' });

  userEvent.click(screen.getByRole('tab', { name: /size/i }));

  screen.getByRole('radiogroup', { name: 'Text Size Settings' });
});

test('renders "Audio/Video Only" tab when enabled', () => {
  render(<VoterSettings onClose={jest.fn()} allowAudioVideoOnlyToggles />);

  userEvent.click(screen.getByRole('tab', { name: /Audio\/Video Only/i }));
  screen.getByRole('button', { name: 'Enable Audio-Only Mode' });
});

test('does not render "Audio/Video Only" tab when not enabled', () => {
  render(
    <VoterSettings onClose={jest.fn()} allowAudioVideoOnlyToggles={false} />
  );

  expect(screen.queryByText('Audio/Video Only')).toBeNull();
});

test('resets button resets global theme', () => {
  let currentTheme: UiTheme | null = null;

  function TestComponent(): JSX.Element {
    return (
      <ThemeConsumer>
        {(theme) => {
          currentTheme = theme;
          return <VoterSettings onClose={jest.fn()} />;
        }}
      </ThemeConsumer>
    );
  }

  render(<TestComponent />, {
    vxTheme: { colorMode: 'contrastHighDark', sizeMode: 'touchLarge' },
  });

  expect(currentTheme).toEqual(
    expect.objectContaining<Partial<UiTheme>>({
      colorMode: 'contrastHighDark',
      sizeMode: 'touchLarge',
    })
  );

  userEvent.click(screen.getByRole('tab', { name: /color/i }));
  userEvent.click(screen.getByRole('radio', { name: /gray text/i }));

  userEvent.click(screen.getByRole('tab', { name: /size/i }));
  userEvent.click(screen.getByRole('radio', { name: /small/i }));

  expect(currentTheme).toEqual(
    expect.objectContaining<Partial<UiTheme>>({
      colorMode: 'contrastLow',
      sizeMode: 'touchSmall',
    })
  );

  userEvent.click(screen.getButton('Reset'));

  expect(currentTheme).toEqual(
    expect.objectContaining<Partial<UiTheme>>({
      colorMode: 'contrastHighDark',
      sizeMode: 'touchLarge',
    })
  );
});

test('done button fires onClose event', () => {
  const onClose = jest.fn();
  render(<VoterSettings onClose={onClose} />);

  expect(onClose).not.toHaveBeenCalled();

  userEvent.click(screen.getButton('Done'));

  expect(onClose).toHaveBeenCalled();
});
