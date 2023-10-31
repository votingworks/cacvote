import { LogEventId, LogSource, Logger } from '@votingworks/logging';
import { PORT, RAVE_MARK_WORKSPACE } from './globals';
import * as server from './server';
import { Workspace, createWorkspace } from './workspace';

export type { Api, CreateTestVoterInput } from './app';
export type { AuthStatus } from './types/auth';
export type { ServerId } from './types/db';

const logger = new Logger(LogSource.VxMarkBackend);

function resolveWorkspace(): Workspace {
  const workspacePath = RAVE_MARK_WORKSPACE;
  if (!workspacePath) {
    void logger.log(LogEventId.ScanServiceConfigurationMessage, 'system', {
      message:
        'workspace path could not be determined; pass a workspace or run with MARK_WORKSPACE',
      disposition: 'failure',
    });
    throw new Error(
      'workspace path could not be determined; pass a workspace or run with MARK_WORKSPACE'
    );
  }
  return createWorkspace(workspacePath);
}

function main(): number {
  server.start({
    port: PORT,
    logger,
    workspace: resolveWorkspace(),
  });
  return 0;
}

if (require.main === module) {
  try {
    process.exitCode = main();
  } catch (error) {
    void logger.log(LogEventId.ApplicationStartup, 'system', {
      message: `Error in starting VxMark backend: ${(error as Error).stack}`,
      disposition: 'failure',
    });
    process.exitCode = 1;
  }
}
