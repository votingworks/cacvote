import { LogEventId, Logger } from '@votingworks/logging';
import { ensureDirSync } from 'fs-extra';
import { join, resolve } from 'path';
import { CACVOTE_MARK_WORKSPACE } from './globals';
import { Store } from './store';

export interface Workspace {
  /**
   * The path to the workspace root.
   */
  readonly path: string;

  /**
   * The store associated with the workspace.
   */
  readonly store: Store;

  /**
   * Reset the workspace, including the election configuration. This is the same
   * as deleting the workspace and recreating it.
   */
  reset(): void;
}

export function createWorkspace(
  root: string,
  storeClass: typeof Store
): Workspace {
  const resolvedRoot = resolve(root);
  ensureDirSync(resolvedRoot);

  const dbPath = join(resolvedRoot, 'cacvote-mark.db');
  const store = storeClass.fileStore(dbPath);

  return {
    path: resolvedRoot,
    store,
    reset() {
      store.reset();
    },
  };
}

export async function resolveWorkspace(
  logger: Logger,
  storeClass: typeof Store
): Promise<Workspace> {
  const workspacePath = CACVOTE_MARK_WORKSPACE;
  if (!workspacePath) {
    await logger.log(LogEventId.WorkspaceConfigurationMessage, 'system', {
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
