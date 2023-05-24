/* istanbul ignore file - tested as part of `validate-monorepo` */

import { Result, err, iter, ok } from '@votingworks/basics';
import { readFileSync, writeFileSync } from 'fs';
import { dirname } from 'path';
import { PackageInfo, getWorkspacePackageInfo } from './pnpm';

/**
 * The contents of a VS Code `.code-workspace` file.
 */
export interface WorkspaceConfig {
  folders: WorkspaceFolder[];
  settings?: Record<string, unknown>;
  extensions?: {
    recommendations?: string[];
  };
}

/**
 * An entry in the `folders` value for a VS Code `.code-workspace` file.
 */
export interface WorkspaceFolder {
  path: string;
  name?: string;
}

const DEFAULT_EXTRA_FOLDERS: readonly WorkspaceFolder[] = [
  'docs',
  'libs/usb-mocking',
  'services/converter-ms-sems',
].map((path) => ({ path, name: path }));

function DEFAULT_PACKAGE_FILTER(pkg: PackageInfo) {
  return !pkg.name.startsWith('@types/') && !pkg.name.endsWith('prodserver');
}

/**
 * Error info for when a VS Code `.code-workspace` file fails a check.
 */
export interface UpdateWorkspaceConfigCheckFailed {
  readonly type: 'check-failed';
  readonly updatedConfig: WorkspaceConfig;
  readonly originalConfig: WorkspaceConfig;
}

/**
 * Updates a VS Code `.code-workspace` file.
 */
export function updateWorkspaceConfig(
  filePath: string,
  {
    extraFolders = DEFAULT_EXTRA_FOLDERS,
    pkgFilter = DEFAULT_PACKAGE_FILTER,
    check = false,
  }: {
    extraFolders?: readonly WorkspaceFolder[];
    pkgFilter?: (pkg: PackageInfo) => boolean;
    check?: boolean;
  } = {}
): Result<WorkspaceConfig, UpdateWorkspaceConfigCheckFailed> {
  const originalWorkspaceConfig: WorkspaceConfig = JSON.parse(
    readFileSync(filePath, 'utf-8')
  );

  const updatedConfig: WorkspaceConfig = {
    ...originalWorkspaceConfig,
    folders: [
      ...iter(getWorkspacePackageInfo(dirname(filePath)))
        .filter(([, pkg]) => pkgFilter(pkg))
        .map(([, { relativePath }]) => ({
          path: relativePath,
          name: relativePath,
        })),
      ...extraFolders,
    ].sort((a, b) => a.path.localeCompare(b.path)),
  };

  if (check) {
    if (
      JSON.stringify(originalWorkspaceConfig) !== JSON.stringify(updatedConfig)
    ) {
      return err({
        type: 'check-failed',
        updatedConfig,
        originalConfig: originalWorkspaceConfig,
      });
    }
  } else {
    writeFileSync(filePath, `${JSON.stringify(updatedConfig, null, 2)}\n`);
  }

  return ok(updatedConfig);
}
