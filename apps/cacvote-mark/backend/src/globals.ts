import { NODE_ENV } from '@votingworks/backend';
import { safeParseInt, unsafeParse } from '@votingworks/types';
import * as dotenv from 'dotenv';
import * as dotenvExpand from 'dotenv-expand';
import fs from 'fs';
import { join } from 'path';
import { z } from 'zod';
import { typedAs } from '@votingworks/basics';
import { FileKey, TpmKey } from '@votingworks/auth';

// https://github.com/bkeepers/dotenv#what-other-env-files-can-i-use
const dotEnvPath = '.env';
const dotenvFiles: string[] = [
  `${dotEnvPath}.${NODE_ENV}.local`,
  // Don't include `.env.local` for `test` environment
  // since normally you expect tests to produce the same
  // results for everyone
  NODE_ENV !== 'test' ? `${dotEnvPath}.local` : '',
  `${dotEnvPath}.${NODE_ENV}`,
  dotEnvPath,
  NODE_ENV !== 'test' ? `../../../${dotEnvPath}.local` : '',
  `../../../${dotEnvPath}`,
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

const BASE_PORT = safeParseInt(process.env.BASE_PORT).ok() || 3000;

/**
 * Default port for the CACvoteMark backend.
 */
export const PORT = safeParseInt(process.env.PORT).ok() || BASE_PORT + 2;

/**
 * Where should the database and audio files go?
 */
export const CACVOTE_MARK_WORKSPACE =
  process.env.CACVOTE_MARK_WORKSPACE ??
  (NODE_ENV === 'development'
    ? join(__dirname, '../dev-workspace')
    : undefined);

/**
 * Is this running as part of an integration test?
 */
export const IS_INTEGRATION_TEST = process.env.IS_INTEGRATION_TEST === 'true';

/**
 * CACvote Server URL.
 */
export const CACVOTE_URL = process.env.CACVOTE_URL
  ? new URL(process.env.CACVOTE_URL)
  : NODE_ENV === 'development' || typeof jest !== 'undefined'
  ? new URL('http://localhost:8000')
  : undefined;

/**
 * Signing certificate for VotingWorks.
 */
export const VX_CA_CERT = process.env.VX_CA_CERT
  ? fs.readFileSync(process.env.VX_CA_CERT)
  : undefined;

/**
 * Signing certificates for Common Access Card (CAC) authentication.
 */
export const CAC_ROOT_CA_CERTS = process.env.CAC_ROOT_CA_CERTS
  ? process.env.CAC_ROOT_CA_CERTS.split(',').map((path) =>
      fs.readFileSync(path)
    )
  : undefined;

/**
 * Signing certificate for communicating with the CACvote Server.
 */
export const MACHINE_CERT = process.env.MACHINE_CERT
  ? fs.readFileSync(process.env.MACHINE_CERT)
  : undefined;

/**
 * Signer corresponding to the signing certificate.
 */
export const SIGNER = process.env.SIGNER?.match(/^tpm(:.+)?$/)
  ? typedAs<TpmKey>({ source: 'tpm' })
  : process.env.SIGNER
  ? typedAs<FileKey>({ source: 'file', path: process.env.SIGNER })
  : undefined;

/**
 * What is the path to the election file for the usability test? If this is set,
 * we mock the CACvote Server.
 */
export const { USABILITY_TEST_ELECTION_PATH } = process.env;

/**
 * True if the app should run in usability test mode.
 */
export const IS_RUNNING_USABILITY_TEST = !!USABILITY_TEST_ELECTION_PATH;

/**
 * Should the usability test flow skip registration or not?
 */
export const USABILITY_TEST_SKIP_REGISTRATION = ['true', 'TRUE', '1'].includes(
  process.env.USABILITY_TEST_SKIP_REGISTRATION ?? 'false'
);

/**
 * Where is the `libNPrint` wrapper binary located?
 */
export const LIBNPRINT_WRAPPER_PATH = unsafeParse(
  z.string(),
  process.env.LIBNPRINT_WRAPPER_PATH
);
