import { safeParseInt } from '@votingworks/types';
import { defineConfig } from 'cypress';

const basePort = safeParseInt(process.env['BASE_PORT']).ok() ?? 3000;
const port = safeParseInt(process.env['PORT']).ok() ?? basePort;

export default defineConfig({
  e2e: {
    baseUrl: `http://localhost:${port}`,
  },

  viewportWidth: 1920,
  viewportHeight: 1080,
});
