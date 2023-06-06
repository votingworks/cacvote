import { LogEventId, Logger } from '@votingworks/logging';
import { Server } from 'http';
import { buildApp } from './app';

export interface StartOptions {
  logger: Logger;
  port: number | string;
}

/**
 * Starts the server with all the default options.
 */
export function start({ logger, port }: StartOptions): Server {
  const app = buildApp();

  return app.listen(
    port,
    /* istanbul ignore next */
    async () => {
      await logger.log(LogEventId.ApplicationStartup, 'system', {
        message: `VxMark backend running at http://localhost:${port}/`,
        disposition: 'success',
      });
    }
  );
}
