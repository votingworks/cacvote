import {
  ArtifactAuthenticatorApi,
  buildMockArtifactAuthenticator,
  buildMockInsertedSmartCardAuth,
  InsertedSmartCardAuthApi,
} from '@votingworks/auth';
import { createMockUsb, MockUsb } from '@votingworks/backend';
import * as grout from '@votingworks/grout';
import { Application } from 'express';
import { Server } from 'http';
import { AddressInfo } from 'net';
import { Api, buildApp } from '../src/app';

interface MockAppContents {
  apiClient: grout.Client<Api>;
  app: Application;
  mockAuth: InsertedSmartCardAuthApi;
  mockArtifactAuthenticator: ArtifactAuthenticatorApi;
  mockUsb: MockUsb;
  server: Server;
}

export function createApp(): MockAppContents {
  const mockAuth = buildMockInsertedSmartCardAuth();
  const mockArtifactAuthenticator = buildMockArtifactAuthenticator();
  const mockUsb = createMockUsb();

  const app = buildApp();

  const server = app.listen();
  const { port } = server.address() as AddressInfo;
  const baseUrl = `http://localhost:${port}/api`;

  const apiClient = grout.createClient<Api>({ baseUrl });

  return {
    apiClient,
    app,
    mockAuth,
    mockArtifactAuthenticator,
    mockUsb,
    server,
  };
}
