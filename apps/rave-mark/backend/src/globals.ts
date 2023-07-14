import { safeParseInt, unsafeParse } from '@votingworks/types';
import { join } from 'path';
import { z } from 'zod';

const BASE_PORT = safeParseInt(process.env.BASE_PORT).ok() || 3000;

/**
 * Default port for the RaveMark backend.
 */
export const PORT = safeParseInt(process.env.PORT).ok() || BASE_PORT + 2;

const NodeEnvSchema = z.union([
  z.literal('development'),
  z.literal('test'),
  z.literal('production'),
]);

/**
 * Which node environment is this?
 */
export const NODE_ENV = unsafeParse(
  NodeEnvSchema,
  process.env.NODE_ENV ?? 'development'
);

/**
 * Where should the database and audio files go?
 */
export const RAVE_MARK_WORKSPACE =
  process.env.RAVE_MARK_WORKSPACE ??
  (NODE_ENV === 'development'
    ? join(__dirname, '../dev-workspace')
    : undefined);

/**
 * Is this running as part of an integration test?
 */
export const IS_INTEGRATION_TEST = process.env.IS_INTEGRATION_TEST === 'true';

/**
 * Should we mock the RAVE server?
 */
export const USE_MOCK_RAVE_SERVER =
  (process.env.USE_MOCK_RAVE_SERVER === 'true' || IS_INTEGRATION_TEST) &&
  process.env.USE_MOCK_RAVE_SERVER !== 'false';
