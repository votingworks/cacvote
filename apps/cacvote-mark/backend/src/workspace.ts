import { ensureDirSync } from 'fs-extra';
import { join, resolve } from 'path';
import { LogEventId, Logger } from '@votingworks/logging';
import { Store } from './store';
import { CACVOTE_MARK_WORKSPACE } from './globals';

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

export function createWorkspace(root: string): Workspace {
  const resolvedRoot = resolve(root);
  ensureDirSync(resolvedRoot);

  const dbPath = join(resolvedRoot, 'cacvote-mark.db');
  const store = Store.fileStore(dbPath);

  return {
    path: resolvedRoot,
    store,
    reset() {
      store.reset();
    },
  };
}

export async function resolveWorkspace(logger?: Logger): Promise<Workspace> {
  const workspacePath = CACVOTE_MARK_WORKSPACE;
  if (!workspacePath) {
    await logger?.log(LogEventId.ScanServiceConfigurationMessage, 'system', {
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
