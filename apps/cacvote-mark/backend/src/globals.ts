import { NODE_ENV } from '@votingworks/backend';
import { safeParseInt, unsafeParse } from '@votingworks/types';
import * as dotenv from 'dotenv';
import * as dotenvExpand from 'dotenv-expand';
import fs from 'fs';
import { join } from 'path';
import { z } from 'zod';
import { typedAs } from '@votingworks/basics';
import { FileKey, TpmKey } from '@votingworks/auth';
import { AutomaticExpirationTypeSchema } from './cacvote-server/usability_test_client';

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
 * Default port for the RaveMark backend.
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
 * Signing certificate for communicating with the CACvote Server.
 */
export const CA_CERT = process.env.CA_CERT
  ? fs.readFileSync(process.env.CA_CERT)
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
 * How many minutes should we wait before expiring a completed voting session
 * during the usability test?
 */
export const USABILITY_TEST_EXPIRATION_MINUTES =
  safeParseInt(process.env.USABILITY_TEST_EXPIRATION_MINUTES, {
    min: 1,
  }).ok() ?? 2;

/**
 * Which type of automatic expiration should we use for the usability test?
 */
export const USABILITY_TEST_AUTOMATIC_EXPIRATION_TYPE =
  AutomaticExpirationTypeSchema.optional().parse(
    process.env.USABILITY_TEST_EXPIRATION_TYPE
  ) ?? 'castBallotOnly';

/**
 * What is the name of the mailing label printer?
 */
export const MAILING_LABEL_PRINTER = unsafeParse(
  z.string(),
  process.env.MAILING_LABEL_PRINTER
);
