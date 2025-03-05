import { mockLogger } from '@votingworks/logging';
import { rmSync } from 'fs-extra';
import { dirSync } from 'tmp';
import { start } from './server';
import { createWorkspace } from './workspace';
import { buildMockAuth } from '../test/app_helpers';

let workdir: string;

beforeEach(() => {
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
    workspace: createWorkspace(workdir),
  });
  expect(server.listening).toBeTruthy();
  server.close();
});
