import { mockLogger } from '@votingworks/logging';
import { rmSync } from 'fs-extra';
import { dirSync } from 'tmp';
import {
  BooleanEnvironmentVariableName,
  getFeatureFlagMock,
} from '@votingworks/utils';
import { start } from './server';
import { createWorkspace } from './workspace';
import { buildMockAuth } from '../test/app_helpers';
import { Store } from './store';

let workdir: string;

const mockFeatureFlagger = getFeatureFlagMock();

jest.mock('@votingworks/utils', (): typeof import('@votingworks/utils') => ({
  ...jest.requireActual('@votingworks/utils'),
  isFeatureFlagEnabled: (flag) => mockFeatureFlagger.isEnabled(flag),
}));

beforeEach(() => {
  mockFeatureFlagger.enableFeatureFlag(
    BooleanEnvironmentVariableName.USE_MOCK_PRINTER
  );
  workdir = dirSync().name;
});

afterEach(() => {
  rmSync(workdir, { recursive: true });
  workdir = '';
});

test('can start server', async () => {
  const logger = mockLogger();
  const auth = buildMockAuth();

  const server = await start({
    auth,
    logger,
    port: 0,
    workspace: createWorkspace(workdir, Store),
  });
  expect(server.listening).toBeTruthy();
  server.close();
});
