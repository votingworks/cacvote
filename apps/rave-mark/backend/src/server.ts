import {
  InsertedSmartCardAuth,
  InsertedSmartCardAuthApi,
  JavaCard,
  MockFileCard,
} from '@votingworks/auth';
import { LogEventId, Logger } from '@votingworks/logging';
import { Server } from 'http';
import {
  BooleanEnvironmentVariableName,
  isFeatureFlagEnabled,
  isIntegrationTest,
} from '@votingworks/utils';
import { buildApp } from './app';
import { Workspace } from './workspace';

export interface StartOptions {
  auth?: InsertedSmartCardAuthApi;
  workspace: Workspace;
  logger: Logger;
  port: number;
}

/**
 * Starts the server with all the default options.
 */
export function start({ auth, logger, port, workspace }: StartOptions): Server {
  const app = buildApp({
    auth:
      auth ??
      new InsertedSmartCardAuth({
        card:
          isFeatureFlagEnabled(BooleanEnvironmentVariableName.USE_MOCK_CARDS) ||
          isIntegrationTest()
            ? new MockFileCard()
            : new JavaCard(),
        config: {
          allowCardlessVoterSessions: true,
          allowElectionManagersToAccessMachinesConfiguredForOtherElections:
            true,
        },
        logger,
      }),

    workspace,
  });

  return app.listen(
    port,
    /* istanbul ignore next */
    async () => {
      await logger.log(LogEventId.ApplicationStartup, 'system', {
        message: `RaveMark backend running at http://localhost:${port}/`,
        disposition: 'success',
      });
    }
  );
}
