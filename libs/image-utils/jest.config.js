const shared = require('../../jest.config.shared');

/**
 * @type {import('@jest/types').Config.InitialOptions}
 */
module.exports = {
  ...shared,
  coveragePathIgnorePatterns: [
    'src/jest_pdf_snapshot.ts',
    'src/cli/pdf_to_images.ts',
  ],
  coverageThreshold: {
    global: {
      statements: 0,
      branches: 0,
      lines: 0,
      functions: 0,
    },
  },
  setupFilesAfterEnv: ['<rootDir>/test/setupTests.ts'],
};
