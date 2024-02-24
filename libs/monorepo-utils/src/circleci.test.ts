import { join } from 'path';
import { iter } from '@votingworks/basics';
import { generateConfig } from './circleci';
import { getWorkspacePackageInfo } from './pnpm';
import { getCargoCrates } from '.';

test('generateConfig', async () => {
  const root = join(__dirname, '../../..');
  const config = iter(
    generateConfig(getWorkspacePackageInfo(root), await getCargoCrates(root))
  ).join();
  expect(config).toContain('test-libs-basics');
  expect(config).toContain('test-crate-controllerd');
});
