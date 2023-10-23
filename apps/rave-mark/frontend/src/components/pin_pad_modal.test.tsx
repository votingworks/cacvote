import { PinLength, renderWithThemes as render } from '@votingworks/ui';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { PinPadModal } from './pin_pad_modal';

const pinLength = PinLength.exactly(8);

test('shows errors if provided', async () => {
  const onEnter = jest.fn();
  const onDismiss = jest.fn();

  const { rerender } = render(
    <PinPadModal
      pinLength={pinLength}
      primaryButtonLabel="Enter"
      dismissButtonLabel="Cancel"
      onEnter={onEnter}
      onDismiss={onDismiss}
      error="Invalid PIN"
    />
  );

  await screen.findByText('Error: Invalid PIN');

  rerender(
    <PinPadModal
      pinLength={pinLength}
      primaryButtonLabel="Enter"
      dismissButtonLabel="Cancel"
      onEnter={onEnter}
      onDismiss={onDismiss}
    />
  );

  expect(screen.queryByText(/Error:/)).not.toBeInTheDocument();
});

test('calls onEnter with the PIN when the enter button is pressed', async () => {
  const onEnter = jest.fn();
  const onDismiss = jest.fn();

  render(
    <PinPadModal
      pinLength={pinLength}
      primaryButtonLabel="Enter"
      dismissButtonLabel="Cancel"
      onEnter={onEnter}
      onDismiss={onDismiss}
    />
  );

  userEvent.click(await screen.findByRole('button', { name: 'enter' }));
  expect(onEnter).toHaveBeenCalledWith('');

  userEvent.click(screen.getByRole('button', { name: '1' }));
  userEvent.click(screen.getByRole('button', { name: '2' }));
  userEvent.click(screen.getByRole('button', { name: '3' }));
  userEvent.click(screen.getByRole('button', { name: '4' }));

  userEvent.click(await screen.findByRole('button', { name: 'enter' }));
  expect(onEnter).toHaveBeenCalledWith('1234');
});

test('calls onDismiss when the dismiss button is pressed', async () => {
  const onEnter = jest.fn();
  const onDismiss = jest.fn();

  render(
    <PinPadModal
      pinLength={pinLength}
      primaryButtonLabel="Enter"
      dismissButtonLabel="Cancel"
      onEnter={onEnter}
      onDismiss={onDismiss}
    />
  );

  userEvent.click(await screen.findByRole('button', { name: 'cancel' }));
  expect(onDismiss).toHaveBeenCalled();
});

test('displays the masked PIN', async () => {
  const onEnter = jest.fn();
  const onDismiss = jest.fn();

  render(
    <PinPadModal
      pinLength={pinLength}
      primaryButtonLabel="Enter"
      dismissButtonLabel="Cancel"
      onEnter={onEnter}
      onDismiss={onDismiss}
    />
  );

  userEvent.click(screen.getByRole('button', { name: '1' }));
  userEvent.click(screen.getByRole('button', { name: '1' }));
  userEvent.click(screen.getByRole('button', { name: '1' }));
  userEvent.click(screen.getByRole('button', { name: '1' }));

  await screen.findByText('• • • • - - - -');
});

test('clears the PIN when the clear button is pressed', async () => {
  const onEnter = jest.fn();
  const onDismiss = jest.fn();

  render(
    <PinPadModal
      pinLength={pinLength}
      primaryButtonLabel="Enter"
      dismissButtonLabel="Cancel"
      onEnter={onEnter}
      onDismiss={onDismiss}
    />
  );

  userEvent.click(screen.getByRole('button', { name: '1' }));
  userEvent.click(screen.getByRole('button', { name: '2' }));
  userEvent.click(screen.getByRole('button', { name: '3' }));
  userEvent.click(screen.getByRole('button', { name: '4' }));
  userEvent.click(screen.getByRole('button', { name: 'clear' }));

  await screen.findByText('- - - - - - - -');
});

test('removes the last digit when the backspace button is pressed', async () => {
  const onEnter = jest.fn();
  const onDismiss = jest.fn();

  render(
    <PinPadModal
      pinLength={pinLength}
      primaryButtonLabel="Enter"
      dismissButtonLabel="Cancel"
      onEnter={onEnter}
      onDismiss={onDismiss}
    />
  );

  userEvent.click(screen.getByRole('button', { name: '1' }));
  userEvent.click(screen.getByRole('button', { name: '2' }));
  userEvent.click(screen.getByRole('button', { name: '3' }));
  userEvent.click(screen.getByRole('button', { name: '4' }));
  userEvent.click(screen.getByRole('button', { name: 'backspace' }));

  await screen.findByText('• • • - - - - -');
});
