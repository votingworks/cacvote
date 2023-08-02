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
import { RAVE_URL, USE_MOCK_RAVE_SERVER } from './globals';
import {
  MockRaveServerClient,
  RaveServerClient,
  RaveServerClientImpl,
} from './rave_server_client';
import { Workspace } from './workspace';

export interface StartOptions {
  auth?: InsertedSmartCardAuthApi;
  workspace: Workspace;
  logger: Logger;
  port: number;
}

function getDefaultAuth(logger: Logger): InsertedSmartCardAuthApi {
  return new InsertedSmartCardAuth({
    card:
      isFeatureFlagEnabled(BooleanEnvironmentVariableName.USE_MOCK_CARDS) ||
      isIntegrationTest()
        ? new MockFileCard()
        : new JavaCard(),
    config: {
      allowCardlessVoterSessions: true,
      allowElectionManagersToAccessMachinesConfiguredForOtherElections: true,
    },
    logger,
  });
}

function getRaveServerClient(workspace: Workspace): RaveServerClient {
  if (USE_MOCK_RAVE_SERVER) {
    return new MockRaveServerClient(workspace.store);
  }
  const baseUrl = RAVE_URL;

  if (!baseUrl) {
    throw new Error('RAVE_URL is not set');
  }

  return new RaveServerClientImpl({
    store: workspace.store,
    baseUrl,
  });
}

/**
 * Starts the server with all the default options.
 */
export function start({ auth, logger, port, workspace }: StartOptions): Server {
  const app = buildApp({
    workspace,
    auth: auth ?? getDefaultAuth(logger),
    raveServerClient: getRaveServerClient(workspace),
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
