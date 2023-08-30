import { Buffer } from 'buffer';
import * as grout from '@votingworks/grout';
import { Application } from 'express';
import { Server } from 'http';
import { AddressInfo } from 'net';
import { dirSync } from 'tmp';
import { Api, buildApp } from '../src/app';
import { createWorkspace } from '../src/workspace';
import { MockRaveServerClient } from '../src/rave_server_client';
import { Store } from '../src/store';
import { Auth, AuthStatus } from '../src/types/auth';

interface MockAppContents {
  apiClient: grout.Client<Api>;
  app: Application;
  mockAuth: Auth;
  server: Server;
}

export function buildMockAuth({
  pin: actualPin = '000000',
}: { pin?: string } = {}): Auth {
  let currentStatus: AuthStatus = {
    status: 'logged_out',
  };

  return {
    getAuthStatus() {
      return Promise.resolve(currentStatus);
    },

    checkPin(pin) {
      if (pin === actualPin) {
        currentStatus = {
          status: 'logged_in',
          user: {
            role: 'rave_voter',
            givenName: 'Joe',
            familyName: 'Smith',
            commonAccessCardId: '1234567890',
          },
          isAdmin: false,
        };
        return Promise.resolve(true);
      }

      return Promise.resolve(false);
    },

    generateSignature() {
      return Promise.resolve(Buffer.from('signature'));
    },

    getCertificate() {
      return Promise.resolve(Buffer.from('certificate'));
    },

    logOut() {
      currentStatus = {
        status: 'logged_out',
      };
      return Promise.resolve();
    },
  };
}

export function createApp(): MockAppContents {
  const mockAuth = buildMockAuth();
  const workdir = dirSync().name;

  const app = buildApp({
    auth: mockAuth,
    workspace: createWorkspace(workdir),
    raveServerClient: new MockRaveServerClient(Store.memoryStore()),
  });

  const server = app.listen();
  const { port } = server.address() as AddressInfo;
  const baseUrl = `http://localhost:${port}/api`;

  const apiClient = grout.createClient<Api>({ baseUrl });

  return {
    apiClient,
    app,
    mockAuth,
    server,
  };
}
