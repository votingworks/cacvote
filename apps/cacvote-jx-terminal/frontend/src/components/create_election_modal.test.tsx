import { renderWithThemes } from '@votingworks/ui';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { electionFamousNames2021Fixtures } from '@votingworks/fixtures';
import { CreateElectionModal } from './create_election_modal';

test('happy path', async () => {
  const onCreate = jest.fn();
  const onClose = jest.fn();
  renderWithThemes(
    <CreateElectionModal onCreate={onCreate} onClose={onClose} />
  );

  const createButton = await screen.findByRole('button', {
    name: /Create Election/i,
  });
  userEvent.click(createButton);

  // onCreate should not be called because the modal is not ready to create
  expect(onCreate).not.toHaveBeenCalled();

  const mailingAddressInput = await screen.findByLabelText(/Mailing Address/i);
  userEvent.type(mailingAddressInput, '123 Main St');

  // onCreate should not be called because the modal is not ready to create
  userEvent.click(createButton);
  expect(onCreate).not.toHaveBeenCalled();

  const fileInput = await screen.findByLabelText(/Election Definition/i);
  userEvent.upload(
    fileInput,
    new File(
      [electionFamousNames2021Fixtures.electionDefinition.electionData],
      'election-definition.json'
    )
  );

  // reading the file is async, so we need to wait for it to be read
  await waitFor(() => {
    // onCreate should be called because the modal is ready to create
    userEvent.click(createButton);

    expect(onCreate).toHaveBeenCalledWith({
      mailingAddress: '123 Main St',
      electionDefinition: electionFamousNames2021Fixtures.electionDefinition,
    });
  });
});

test('cancel button', async () => {
  const onCreate = jest.fn();
  const onClose = jest.fn();
  renderWithThemes(
    <CreateElectionModal onCreate={onCreate} onClose={onClose} />
  );

  const cancelButton = await screen.findByRole('button', { name: /Cancel/i });
  userEvent.click(cancelButton);

  expect(onClose).toHaveBeenCalled();
});

test('enter key', async () => {
  const onCreate = jest.fn();
  const onClose = jest.fn();
  renderWithThemes(
    <CreateElectionModal onCreate={onCreate} onClose={onClose} />
  );

  const mailingAddressInput = await screen.findByLabelText(/Mailing Address/i);
  userEvent.type(mailingAddressInput, '123 Main St');

  const fileInput = await screen.findByLabelText(/Election Definition/i);
  userEvent.upload(
    fileInput,
    new File(
      [electionFamousNames2021Fixtures.electionDefinition.electionData],
      'election-definition.json'
    )
  );

  await waitFor(() => {
    const createButton = screen.getByRole('button', {
      name: /Create Election/i,
    });
    userEvent.type(createButton, '{enter}');

    expect(onCreate).toHaveBeenCalledWith({
      mailingAddress: '123 Main St',
      electionDefinition: electionFamousNames2021Fixtures.electionDefinition,
    });
  });
});

test('escape key', async () => {
  const onCreate = jest.fn();
  const onClose = jest.fn();
  renderWithThemes(
    <CreateElectionModal onCreate={onCreate} onClose={onClose} />
  );

  const cancelButton = await screen.findByRole('button', { name: /Cancel/i });
  userEvent.type(cancelButton, '{esc}');

  expect(onClose).toHaveBeenCalled();
});
