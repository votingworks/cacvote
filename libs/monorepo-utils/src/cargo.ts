import { spawn } from 'child_process';
import { lines } from '@votingworks/basics';
import { relative } from 'path';
import { getAbsoluteRootPath } from './dependencies';

/**
 * Information about Rust cargo crates in the monorepo.
 */
export interface CargoCrate {
  name: string;
  version: string;
  absolutePath: string;
  workspacePath: string;
}

/**
 * Get all Rust crate info.
 */
export function getCargoCrates(root: string): Promise<CargoCrate[]> {
  const absoluteRootPath = getAbsoluteRootPath(root);
  // Output is formatted like
  // "package-id v0.1.2 (/path/to/package)"
  // <newline>
  // "another-package-id v3.4.5 (/path/to/other-package)"
  const { stdout } = spawn(
    'cargo',
    ['tree', '-e', 'no-normal', '-e', 'no-dev', '-e', 'no-build'],
    { cwd: absoluteRootPath }
  );

  return lines(stdout)
    .filterMap((line) => {
      const match = line.match(/^(\S+) v(\S+)(?: \(proc-macro\))? \((.+)\)$/);
      if (!match) {
        return;
      }

      const name = match[1] as string;
      const version = match[2] as string;
      const absolutePath = match[3] as string;

      const workspacePath = relative(absoluteRootPath, absolutePath);
      return { name, version, absolutePath, workspacePath };
    })
    .toArray();
}
