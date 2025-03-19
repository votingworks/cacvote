import * as grout from '@votingworks/grout';
import {
  BooleanEnvironmentVariableName,
  getFeatureFlagMock,
} from '@votingworks/utils';
import { Server } from 'http';
import { createApp } from '../test/app_helpers';
import { Api } from './app';

let apiClient: grout.Client<Api>;
let server: Server;

const mockFeatureFlagger = getFeatureFlagMock();

jest.mock('@votingworks/utils', (): typeof import('@votingworks/utils') => ({
  ...jest.requireActual('@votingworks/utils'),
  isFeatureFlagEnabled: (flag) => mockFeatureFlagger.isEnabled(flag),
}));

beforeEach(async () => {
  mockFeatureFlagger.enableFeatureFlag(
    BooleanEnvironmentVariableName.USE_MOCK_PRINTER
  );
  ({ apiClient, server } = await createApp());
});

afterEach(() => {
  server?.close();
});

test('has an API client', () => {
  expect(apiClient).toBeDefined();
});
