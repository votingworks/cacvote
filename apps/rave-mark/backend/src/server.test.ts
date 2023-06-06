import { fakeLogger } from '@votingworks/logging';
import { PORT } from './globals';
import { start } from './server';

test('can start server', () => {
  const logger = fakeLogger();

  const server = start({
    logger,
    port: PORT,
  });
  expect(server.listening).toBeTruthy();
  server.close();
});
