import * as grout from '@votingworks/grout';
import { Server } from 'http';
import { createApp } from '../test/app_helpers';
import { Api } from './app';

let apiClient: grout.Client<Api>;
let server: Server;

beforeEach(() => {
  ({ apiClient, server } = createApp());
});

afterEach(() => {
  server?.close();
});

test('has an API client', () => {
  expect(apiClient).toBeDefined();
});
