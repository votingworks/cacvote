import { join } from 'path';
import { existsSync } from 'fs';
import { iter } from '@votingworks/basics';
import { PnpmPackageInfo } from './types';
import { CargoCrate } from './cargo';

function jobIdForPackage(pkg: PnpmPackageInfo): string {
  return `test-${pkg.relativePath.replace(/\//g, '-')}`;
}

function jobIdForRustCrate(crate: CargoCrate): string {
  return `test-crate-${crate.name}`;
}

function* generateTestJobForNodeJsPackage(
  pkg: PnpmPackageInfo
): Iterable<string> {
  const hasPlaywrightTests = existsSync(`${pkg.path}/playwright.config.ts`);
  const isIntegrationTestJob = hasPlaywrightTests;
  /* istanbul ignore next */
  yield `# ${pkg.name}\n`;
  yield `${jobIdForPackage(pkg)}:\n`;

  /* istanbul ignore next */
  yield `  executor: ${isIntegrationTestJob ? 'nodejs-browsers' : 'nodejs'}\n`;
  yield `  resource_class: xlarge\n`;
  yield `  steps:\n`;
  yield `    - checkout-and-install\n`;

  /* istanbul ignore next */
  if (hasPlaywrightTests) {
    yield `    - run:\n`;
    yield `        name: Install Browser\n`;
    yield `        command: |\n`;
    yield `          pnpm --dir ${pkg.relativePath} exec playwright install-deps\n`;
    yield `          pnpm --dir ${pkg.relativePath} exec playwright install chromium\n`;
  }

  yield `    - run:\n`;
  yield `        name: Build\n`;
  yield `        command: |\n`;
  yield `          pnpm --dir ${pkg.relativePath} build\n`;
  yield `    - run:\n`;
  yield `        name: Lint\n`;
  yield `        command: |\n`;
  yield `          pnpm --dir ${pkg.relativePath} lint\n`;
  yield `    - run:\n`;
  yield `        name: Test\n`;
  yield `        command: |\n`;
  yield `          pnpm --dir ${pkg.relativePath} test\n`;
  yield `        environment:\n`;
  yield `          JEST_JUNIT_OUTPUT_DIR: ./reports/\n`;
  yield `    - store_test_results:\n`;
  yield `        path: ${pkg.relativePath}/reports/\n`;

  /* istanbul ignore next */
  if (hasPlaywrightTests) {
    yield `    - store_artifacts:\n`;
    yield `        path: ${pkg.relativePath}/test-results/\n`;
  }
}

function* generateTestJobForRustCrate(crate: CargoCrate): Iterable<string> {
  const hasDatabase = existsSync(join(crate.absolutePath, 'db/migrations'));
  const databaseUrl = hasDatabase
    ? `postgresql://root@localhost:5432/${crate.name}`
    : '';

  yield `${jobIdForRustCrate(crate)}:\n`;
  yield `  executor: ${hasDatabase ? 'rust-db' : 'nodejs'}\n`;
  yield `  resource_class: xlarge\n`;
  yield `  steps:\n`;
  yield `    - checkout-and-install\n`;

  if (hasDatabase) {
    yield `    - run:\n`;
    yield `        name: Setup Database\n`;
    yield `        environment:\n`;
    yield `          DATABASE_URL: ${databaseUrl}\n`;
    yield `        command: |\n`;
    yield `          cargo install sqlx-cli\n`;
    yield `          script/reset-db\n`;
  }

  yield `    - run:\n`;
  yield `        name: Build\n`;
  if (hasDatabase) {
    yield `        environment:\n`;
    yield `          DATABASE_URL: ${databaseUrl}\n`;
  }
  yield `        command: |\n`;
  yield `          cargo build -p ${crate.name}\n`;
  yield `    - run:\n`;
  yield `        name: Test\n`;
  if (hasDatabase) {
    yield `        environment:\n`;
    yield `          DATABASE_URL: ${databaseUrl}\n`;
  }
  yield `        command: |\n`;
  yield `          cargo test -p ${crate.name}\n`;
}

function* generateTestJobForPackage(pkg: PnpmPackageInfo): Iterable<string> {
  /* istanbul ignore next */
  if (!pkg.packageJson) {
    throw new Error(`Unsupported package type: ${pkg.relativePath}`);
  }

  yield* generateTestJobForNodeJsPackage(pkg);
}

/**
 * Path to the CircleCI config file.
 */
export const CIRCLECI_CONFIG_PATH = join(
  __dirname,
  '../../../.circleci/config.yml'
);

/**
 * Generate a CircleCI config file.
 */
export function* generateConfig(
  pnpmPackages: ReadonlyMap<string, PnpmPackageInfo>,
  rustCrates: readonly CargoCrate[]
): Iterable<string> {
  const pnpmJobs = [...pnpmPackages.values()].reduce((memo, pkg) => {
    /* istanbul ignore next */
    if (!pkg.packageJson?.scripts?.['test']) {
      // exclude packages without tests
      return memo;
    }

    const jobLines = generateTestJobForPackage(pkg);
    return memo.set(pkg, jobLines);
  }, new Map<PnpmPackageInfo, Iterable<string>>());
  const rustJobs = rustCrates.map(generateTestJobForRustCrate);
  const jobIds = [
    ...[...pnpmJobs.keys()].map(jobIdForPackage),
    ...iter(rustCrates).map(jobIdForRustCrate),
    // hardcoded jobs
    'validate-monorepo',
  ];

  yield `# THIS FILE IS GENERATED. DO NOT EDIT IT DIRECTLY.\n`;
  yield `# Run \`pnpm -w generate-circleci-config\` to regenerate it.\n`;
  yield `\n`;
  yield `version: 2.1\n`;
  yield `\n`;
  yield `orbs:\n`;
  yield `  browser-tools: circleci/browser-tools@1.4.3\n`;
  yield `\n`;
  yield `executors:\n`;
  yield `  nodejs-browsers:\n`;
  yield `    docker:\n`;
  yield `      - image: votingworks/cimg-debian12-browsers:3.0.1\n`;
  yield `        auth:\n`;
  yield `          username: $VX_DOCKER_USERNAME\n`;
  yield `          password: $VX_DOCKER_PASSWORD\n`;
  yield `  nodejs:\n`;
  yield `    docker:\n`;
  yield `      - image: votingworks/cimg-debian12:3.0.1\n`;
  yield `        auth:\n`;
  yield `          username: $VX_DOCKER_USERNAME\n`;
  yield `          password: $VX_DOCKER_PASSWORD\n`;
  yield `  rust-db:\n`;
  yield `    docker:\n`;
  yield `      - image: votingworks/cimg-debian12:3.0.1\n`;
  yield `        auth:\n`;
  yield `          username: $VX_DOCKER_USERNAME\n`;
  yield `          password: $VX_DOCKER_PASSWORD\n`;
  yield `        environment:\n`;
  yield `          DATABASE_URL: postgresql://root@localhost:5432/cacvote-test\n`;
  yield `      - image: cimg/postgres:15.6\n`;
  yield `        environment:\n`;
  yield `          POSTGRES_USER: root\n`;
  yield `          POSTGRES_DB: cacvote-test\n`;
  yield `\n`;
  yield `jobs:\n`;

  for (const group of [pnpmJobs.values(), rustJobs]) {
    for (const pnpmJob of group) {
      for (const line of pnpmJob) {
        yield `  ${line}`;
      }

      yield '\n';
    }
  }

  yield `  validate-monorepo:\n`;
  yield `    executor: nodejs\n`;
  yield `    resource_class: xlarge\n`;
  yield `    steps:\n`;
  yield `      - checkout-and-install\n`;
  yield `      - run:\n`;
  yield `          name: Build\n`;
  yield `          command: |\n`;
  yield `            pnpm --dir script build\n`;
  yield `      - run:\n`;
  yield `          name: Validate\n`;
  yield `          command: |\n`;
  yield `            ./script/validate-monorepo\n`;
  yield `\n`;
  yield `workflows:\n`;
  yield `  test:\n`;
  yield `    jobs:\n`;

  for (const jobId of jobIds) {
    yield `      - ${jobId}\n`;
  }

  yield `\n`;
  yield `commands:\n`;
  yield `  checkout-and-install:\n`;
  yield `    description: Get the code and install dependencies.\n`;
  yield `    steps:\n`;
  yield `      - run:\n`;
  yield `          name: Ensure rust is in the PATH variable\n`;
  yield `          command: |\n`;
  yield `            echo 'export PATH="/root/.cargo/bin:$PATH"' >> $BASH_ENV\n`;
  yield `      - checkout\n`;
  yield `      # Edit this comment somehow in order to invalidate the CircleCI cache.\n`;
  yield `      # Since the contents of this file affect the cache key, editing only a\n`;
  yield `      # comment will invalidate the cache without changing the behavior.\n`;
  yield `      # last edited by Ben 2023-11-17\n`;
  yield `      - restore_cache:\n`;
  yield `          key:\n`;
  yield `            dotcache-cache-{{checksum ".circleci/config.yml" }}-{{ checksum\n`;
  yield `            "pnpm-lock.yaml" }}\n`;
  yield `      - run:\n`;
  yield `          name: Install OpenSSL\n`;
  yield `          command: |\n`;
  yield `            apt-get update\n`;
  yield `            apt-get install libssl-dev -y\n`;
  yield `      - run:\n`;
  yield `          name: Update Rust\n`;
  yield `          command: |\n`;
  yield `            rustup update stable\n`;
  yield `            rustup default stable\n`;
  yield `      - run:\n`;
  yield `          name: Setup Dependencies\n`;
  yield `          command: |\n`;
  yield `            pnpm install --frozen-lockfile\n`;
  yield `            pnpm --recursive install:rust-addon\n`;
  yield `            pnpm --recursive build:rust-addon\n`;
  yield `      - save_cache:\n`;
  yield `          key:\n`;
  yield `            dotcache-cache-{{checksum ".circleci/config.yml" }}-{{ checksum\n`;
  yield `            "pnpm-lock.yaml" }}\n`;
  yield `          paths:\n`;
  yield `            - /root/.local/share/pnpm/store/v3\n`;
  yield `            - /root/.cache/ms-playwright\n`;
  yield `            - /root/.cargo\n`;
}
