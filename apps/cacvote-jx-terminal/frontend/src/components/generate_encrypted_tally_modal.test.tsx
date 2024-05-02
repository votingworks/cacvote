import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithThemes } from '@votingworks/ui';
import { deferred } from '@votingworks/basics';
import { GenerateEncryptedTallyModal } from './generate_encrypted_tally_modal';

test('shows voter & ballot counts', async () => {
  renderWithThemes(
    <GenerateEncryptedTallyModal
      onGenerate={() => Promise.resolve()}
      onClose={() => {}}
      registeredVoterCount={100}
      castBallotCount={200}
    />
  );

  await screen.findByText(/100 registered voters/i);
  await screen.findByText(/200 cast ballots/i);
});

test('calls onGenerate when confirmed', async () => {
  const onGenerate = jest.fn();
  renderWithThemes(
    <GenerateEncryptedTallyModal
      onGenerate={onGenerate}
      onClose={() => {}}
      registeredVoterCount={100}
      castBallotCount={200}
    />
  );

  await screen.findByText(/Would you like to proceed?/i);
  const generateButton = await screen.findByRole('button', {
    name: /generate encrypted tally/i,
  });

  userEvent.click(generateButton);
  expect(onGenerate).toHaveBeenCalled();
});

test('calls onClose after onGenerate resolves', async () => {
  const generateDeferred = deferred<void>();
  const onGenerate = jest.fn().mockReturnValue(generateDeferred.promise);
  const onClose = jest.fn();
  renderWithThemes(
    <GenerateEncryptedTallyModal
      onGenerate={onGenerate}
      onClose={onClose}
      registeredVoterCount={100}
      castBallotCount={200}
    />
  );

  await screen.findByText(/Would you like to proceed?/i);
  const generateButton = await screen.findByRole('button', {
    name: /generate encrypted tally/i,
  });

  userEvent.click(generateButton);
  expect(onGenerate).toHaveBeenCalled();

  generateDeferred.resolve();
  await waitFor(() => expect(onClose).toHaveBeenCalled());
});

test('calls onClose when canceled', async () => {
  const onClose = jest.fn();
  renderWithThemes(
    <GenerateEncryptedTallyModal
      onGenerate={() => Promise.resolve()}
      onClose={onClose}
      registeredVoterCount={100}
      castBallotCount={200}
    />
  );

  await screen.findByText(/Would you like to proceed?/i);
  const cancelButton = await screen.findByRole('button', {
    name: /cancel/i,
  });

  userEvent.click(cancelButton);
  expect(onClose).toHaveBeenCalled();
});

test('does not call onGenerate when generating', async () => {
  const onGenerate = jest.fn();
  renderWithThemes(
    <GenerateEncryptedTallyModal
      onGenerate={onGenerate}
      onClose={() => {}}
      registeredVoterCount={100}
      castBallotCount={200}
      isGenerating
    />
  );

  await screen.findByText(/Generating Encrypted Tally…/i);
  const generateButton = await screen.findByRole('button', {
    name: /generating encrypted tally/i,
  });

  userEvent.click(generateButton);
  expect(onGenerate).not.toHaveBeenCalled();

  userEvent.keyboard('{enter}');
  expect(onGenerate).not.toHaveBeenCalled();
});

test('does not call onClose when generating', async () => {
  const onClose = jest.fn();
  renderWithThemes(
    <GenerateEncryptedTallyModal
      onGenerate={() => Promise.resolve()}
      onClose={onClose}
      registeredVoterCount={100}
      castBallotCount={200}
      isGenerating
    />
  );

  const cancelButton = await screen.findByRole('button', {
    name: /cancel/i,
  });

  userEvent.click(cancelButton);
  expect(onClose).not.toHaveBeenCalled();

  userEvent.keyboard('{esc}');
  expect(onClose).not.toHaveBeenCalled();
});

test('calls onGenerate on enter', async () => {
  const onGenerate = jest.fn();
  renderWithThemes(
    <GenerateEncryptedTallyModal
      onGenerate={onGenerate}
      onClose={() => {}}
      registeredVoterCount={100}
      castBallotCount={200}
    />
  );

  await screen.findByText(/Would you like to proceed?/i);
  userEvent.keyboard('{enter}');
  expect(onGenerate).toHaveBeenCalled();
});

test('calls onClose on escape', async () => {
  const onClose = jest.fn();
  renderWithThemes(
    <GenerateEncryptedTallyModal
      onGenerate={() => Promise.resolve()}
      onClose={onClose}
      registeredVoterCount={100}
      castBallotCount={200}
    />
  );

  await screen.findByText(/Would you like to proceed?/i);
  userEvent.keyboard('{esc}');
  expect(onClose).toHaveBeenCalled();
});
