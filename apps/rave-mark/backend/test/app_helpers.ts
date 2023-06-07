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
import { dirSync } from 'tmp';
import { Api, buildApp } from '../src/app';
import { createWorkspace } from '../src/workspace';

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
  const workdir = dirSync().name;

  const app = buildApp({
    auth: mockAuth,
    workspace: createWorkspace(workdir),
  });

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
