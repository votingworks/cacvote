import { buildMockInsertedSmartCardAuth } from '@votingworks/auth';
import { fakeLogger } from '@votingworks/logging';
import { rmSync } from 'fs-extra';
import { dirSync } from 'tmp';
import { PORT } from './globals';
import { start } from './server';
import { createWorkspace } from './workspace';

let workdir: string;

beforeEach(() => {
  workdir = dirSync().name;
});

afterEach(() => {
  rmSync(workdir, { recursive: true });
  workdir = '';
});

test('can start server', () => {
  const logger = fakeLogger();
  const auth = buildMockInsertedSmartCardAuth();

  const server = start({
    auth,
    logger,
    port: PORT,
    workspace: createWorkspace(workdir),
  });
  expect(server.listening).toBeTruthy();
  server.close();
});
