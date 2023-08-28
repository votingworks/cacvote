import { DateTime } from 'luxon';
import userEvent from '@testing-library/user-event';
import {
  fakeSystemAdministratorUser,
  hasTextAcrossElements,
} from '@votingworks/test-utils';
import { DippedSmartCardAuth } from '@votingworks/types';

import { deferred } from '@votingworks/basics';
import { act, render, screen, waitFor } from '../test/react_testing_library';
import { UnlockMachineScreen } from './unlock_machine_screen';
import { PinLength } from './utils/pin_length';

beforeEach(() => {
  jest.useFakeTimers().setSystemTime(new Date('2000-01-01T00:00:00Z'));
});

const checkingPinAuthStatus: DippedSmartCardAuth.CheckingPin = {
  status: 'checking_pin',
  user: fakeSystemAdministratorUser(),
};

test('PIN submission', async () => {
  const checkPin = jest.fn();
  render(
    <UnlockMachineScreen
      auth={checkingPinAuthStatus}
      checkPin={checkPin}
      pinLength={PinLength.exactly(6)}
    />
  );

  screen.getByText('- - - - - -');
  expect(screen.queryByText('Enter')).not.toBeInTheDocument();

  userEvent.click(screen.getButton('0'));
  screen.getByText('• - - - - -');

  userEvent.click(screen.getButton('clear'));
  screen.getByText('- - - - - -');

  userEvent.click(screen.getButton('0'));
  screen.getByText('• - - - - -');

  userEvent.click(screen.getButton('1'));
  screen.getByText('• • - - - -');

  userEvent.click(screen.getButton('2'));
  screen.getByText('• • • - - -');

  userEvent.click(screen.getButton('3'));
  screen.getByText('• • • • - -');

  userEvent.click(screen.getButton('4'));
  screen.getByText('• • • • • -');

  userEvent.click(screen.getButton('backspace'));
  screen.getByText('• • • • - -');

  userEvent.click(screen.getButton('4'));
  screen.getByText('• • • • • -');

  userEvent.click(screen.getButton('5'));
  await waitFor(() => expect(checkPin).toHaveBeenNthCalledWith(1, '012345'));
  await screen.findByText('- - - - - -');
});

test('Incorrect PIN', () => {
  const checkPin = jest.fn();
  render(
    <UnlockMachineScreen
      auth={{
        ...checkingPinAuthStatus,
        wrongPinEnteredAt: new Date(),
      }}
      checkPin={checkPin}
      pinLength={PinLength.exactly(6)}
    />
  );

  screen.getByText('Incorrect PIN. Please try again.');
});

test.each<{
  description: string;
  isWrongPinEnteredAtSet: boolean;
  expectedPromptAfterLockoutEnds: string;
}>([
  {
    description: 'card locked after incorrect PIN attempt',
    isWrongPinEnteredAtSet: true,
    expectedPromptAfterLockoutEnds: 'Incorrect PIN. Please try again.',
  },
  {
    description: 'card already locked',
    isWrongPinEnteredAtSet: false,
    expectedPromptAfterLockoutEnds: 'Enter the card PIN to unlock.',
  },
])(
  'Lockout - $description',
  ({ isWrongPinEnteredAtSet, expectedPromptAfterLockoutEnds }) => {
    const checkPin = jest.fn();
    render(
      <UnlockMachineScreen
        auth={{
          ...checkingPinAuthStatus,
          lockedOutUntil: DateTime.now().plus({ seconds: 60 }).toJSDate(),
          wrongPinEnteredAt: isWrongPinEnteredAtSet ? new Date() : undefined,
        }}
        checkPin={checkPin}
        pinLength={PinLength.exactly(6)}
      />
    );

    screen.getByText(
      hasTextAcrossElements(/Card locked. Please try again in 01m 00s$/)
    );
    screen.getByText('- - - - - -');

    // Ensure number pad entry is ignored
    userEvent.click(screen.getButton('0'));
    screen.getByText('- - - - - -');

    act(() => {
      jest.advanceTimersByTime(1000);
    });
    screen.getByText(
      hasTextAcrossElements(/Card locked. Please try again in 00m 59s$/)
    );

    act(() => {
      jest.advanceTimersByTime(59 * 1000);
    });
    screen.getByText(expectedPromptAfterLockoutEnds);
  }
);

test('Error checking PIN', () => {
  const checkPin = jest.fn();
  render(
    <UnlockMachineScreen
      auth={{
        ...checkingPinAuthStatus,
        error: true,
        wrongPinEnteredAt: new Date(),
      }}
      checkPin={checkPin}
      pinLength={PinLength.exactly(6)}
    />
  );

  screen.getByText('Error checking PIN. Please try again.');
});

test('PIN submission disabled when checking PIN', async () => {
  const checkPin = jest.fn();
  render(
    <UnlockMachineScreen
      auth={{
        ...checkingPinAuthStatus,
        status: 'checking_pin',
      }}
      checkPin={checkPin}
      pinLength={PinLength.exactly(6)}
    />
  );

  const checkPinDeferred = deferred<void>();
  checkPin.mockReturnValueOnce(checkPinDeferred.promise);

  for (let i = 0; i < 6; i += 1) {
    userEvent.click(await screen.findButton('0'));
  }

  expect(checkPin).toHaveBeenCalledTimes(1);
  expect(checkPin).toHaveBeenNthCalledWith(1, '000000');
  expect(await screen.findButton('0')).toBeDisabled();

  // resolve the checkPin promise and let the component re-render
  await act(async () => {
    checkPinDeferred.resolve();
    await Promise.resolve();
  });

  expect(await screen.findButton('0')).toBeEnabled();
});

test('range of acceptable PIN lengths', async () => {
  const checkPin = jest.fn();
  render(
    <UnlockMachineScreen
      auth={checkingPinAuthStatus}
      checkPin={checkPin}
      pinLength={PinLength.range(4, 6)}
    />
  );

  await screen.findByText('- - - - - -');
  userEvent.click(await screen.findButton('0'));

  await screen.findByText('• - - - - -');
  userEvent.click(await screen.findButton('1'));

  await screen.findByText('• • - - - -');
  userEvent.click(await screen.findButton('2'));

  await screen.findByText('• • • - - -');
  userEvent.click(await screen.findButton('3'));

  // submit early by pressing Enter
  await screen.findByText('• • • • - -');
  await act(async () => {
    userEvent.click(await screen.findButton('Enter'));
  });
  expect(checkPin).toHaveBeenCalledWith('0123');

  // re-enter PIN since it gets cleared
  userEvent.click(await screen.findButton('0'));
  userEvent.click(await screen.findButton('1'));
  userEvent.click(await screen.findButton('2'));
  userEvent.click(await screen.findButton('3'));
  userEvent.click(await screen.findButton('4'));

  await screen.findByText('• • • • • -');

  // submit automatically by entering the max number of digits
  await act(async () => {
    userEvent.click(await screen.findButton('5'));
  });
  expect(checkPin).toHaveBeenCalledWith('012345');
});
