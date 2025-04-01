import { LogEventId, LogSource, Logger } from '@votingworks/logging';
import {
  PORT,
  CACVOTE_MARK_WORKSPACE,
  IS_RUNNING_USABILITY_TEST,
} from './globals';
import * as server from './server';
import { Workspace, createWorkspace } from './workspace';
import { Store } from './store';
import { UsabilityTestStore } from './usability-test/usability_test_store';

export type { Api, VoterStatus } from './app';
export type { AuthStatus } from './types/auth';
export type { JurisdictionCode, Uuid } from './cacvote-server/types';

const logger = new Logger(LogSource.VxMarkBackend, () =>
  Promise.resolve('system')
);

function resolveWorkspace(storeClass: typeof Store): Workspace {
  const workspacePath = CACVOTE_MARK_WORKSPACE;
  if (!workspacePath) {
    void logger.log(LogEventId.WorkspaceConfigurationMessage, 'system', {
      message:
        'workspace path could not be determined; pass a workspace or run with MARK_WORKSPACE',
      disposition: 'failure',
    });
    throw new Error(
      'workspace path could not be determined; pass a workspace or run with MARK_WORKSPACE'
    );
  }
  return createWorkspace(workspacePath, storeClass);
}

async function main(): Promise<number> {
  await server.start({
    port: PORT,
    logger,
    workspace: resolveWorkspace(
      IS_RUNNING_USABILITY_TEST ? UsabilityTestStore : Store
    ),
  });
  return 0;
}

if (require.main === module) {
  void (async () => {
    try {
      process.exitCode = await main();
    } catch (error) {
      void logger.log(LogEventId.ApplicationStartup, 'system', {
        message: `Error in starting VxMark backend: ${(error as Error).stack}`,
        disposition: 'failure',
      });
      process.exitCode = 1;
    }
  })();
}
