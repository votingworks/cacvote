import {
  InsertedSmartCardAuth,
  InsertedSmartCardAuthApi,
  JavaCard,
  MockFileCard,
} from '@votingworks/auth';
import { LogEventId, Logger } from '@votingworks/logging';
import {
  BooleanEnvironmentVariableName,
  isFeatureFlagEnabled,
  isIntegrationTest,
} from '@votingworks/utils';
import { Server } from 'http';
import { buildApp } from './app';
import { USE_MOCK_RAVE_SERVER } from './globals';
import {
  MockRaveServerClient,
  RaveServerClientImpl,
} from './rave_server_client';
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

    raveServerClient: USE_MOCK_RAVE_SERVER
      ? new MockRaveServerClient(workspace.store)
      : new RaveServerClientImpl(workspace.store),
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
