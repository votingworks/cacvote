import { join } from 'path';
import { generateConfig } from './circleci';
import { getWorkspacePackageInfo } from './pnpm';
import { getCargoCrates } from '.';

test('generateConfig', async () => {
  const root = join(__dirname, '../../..');
  const config = generateConfig(
    getWorkspacePackageInfo(root),
    await getCargoCrates(root)
  );
  expect(config).toContain('test-libs-basics');
  expect(config).toContain('test-crate-controllerd');
});
