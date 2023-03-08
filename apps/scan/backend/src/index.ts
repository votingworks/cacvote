import { LogEventId, Logger, LogSource } from '@votingworks/logging';
import { createClient } from '@votingworks/plustek-scanner';
import * as dotenv from 'dotenv';
import * as dotenvExpand from 'dotenv-expand';
import fs from 'fs';
import { NODE_ENV, SCAN_WORKSPACE } from './globals';
import { createInterpreter, PrecinctScannerInterpreter } from './interpret';
import * as plustekStateMachine from './state_machine';
import * as server from './server';
import { createWorkspace, Workspace } from './util/workspace';

export type { Api } from './app';
export * from './types';

// https://github.com/bkeepers/dotenv#what-other-env-files-can-i-use
const dotenvPath = '.env';
const dotenvFiles: string[] = [
  `${dotenvPath}.${NODE_ENV}.local`,
  // Don't include `.env.local` for `test` environment
  // since normally you expect tests to produce the same
  // results for everyone
  NODE_ENV !== 'test' ? `${dotenvPath}.local` : '',
  `${dotenvPath}.${NODE_ENV}`,
  dotenvPath,
  NODE_ENV !== 'test' ? `../../${dotenvPath}.local` : '',
  `../../${dotenvPath}`,
].filter(Boolean);

// Load environment variables from .env* files. Suppress warnings using silent
// if this file is missing. dotenv will never modify any environment variables
// that have already been set.  Variable expansion is supported in .env files.
// https://github.com/motdotla/dotenv
// https://github.com/motdotla/dotenv-expand
for (const dotenvFile of dotenvFiles) {
  if (fs.existsSync(dotenvFile)) {
    dotenvExpand.expand(dotenv.config({ path: dotenvFile }));
  }
}

const logger = new Logger(LogSource.VxScanBackend);

async function resolveWorkspace(): Promise<Workspace> {
  const workspacePath = SCAN_WORKSPACE;
  if (!workspacePath) {
    await logger.log(LogEventId.ScanServiceConfigurationMessage, 'system', {
      message:
        'workspace path could not be determined; pass a workspace or run with SCAN_WORKSPACE',
      disposition: 'failure',
    });
    throw new Error(
      'workspace path could not be determined; pass a workspace or run with SCAN_WORKSPACE'
    );
  }
  return await createWorkspace(workspacePath);
}

function createPrecinctScannerStateMachine(
  workspace: Workspace,
  interpreter: PrecinctScannerInterpreter
): plustekStateMachine.PrecinctScannerStateMachine {
  return plustekStateMachine.createPrecinctScannerStateMachine({
    createPlustekClient: createClient,
    workspace,
    interpreter,
    logger,
  });
}

async function main(): Promise<number> {
  const workspace = await resolveWorkspace();
  const precinctScannerInterpreter = createInterpreter();
  const precinctScannerStateMachine = createPrecinctScannerStateMachine(
    workspace,
    precinctScannerInterpreter
  );

  server.start({
    precinctScannerStateMachine,
    precinctScannerInterpreter,
    workspace,
  });

  return 0;
}

if (require.main === module) {
  void main()
    .catch((error) => {
      void logger.log(LogEventId.ApplicationStartup, 'system', {
        message: `Error in starting VxScan backend: ${error.stack}`,
        disposition: 'failure',
      });
      return 1;
    })
    .then((code) => {
      process.exitCode = code;
    });
}