import { ok } from '@votingworks/basics';
import * as grout from '@votingworks/grout';
import { mockLogger } from '@votingworks/logging';
import { Buffer } from 'buffer';
import { Application } from 'express';
import { Server } from 'http';
import { AddressInfo } from 'net';
import { dirSync } from 'tmp';
import { Api, buildApp } from '../src/app';
import { Auth, AuthStatus } from '../src/types/auth';
import { createWorkspace } from '../src/workspace';

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
    status: 'no_card',
  };

  return {
    getAuthStatus() {
      return Promise.resolve(currentStatus);
    },

    checkPin(pin) {
      if (pin === actualPin) {
        currentStatus = {
          status: 'has_card',
          card: {
            givenName: 'Joe',
            familyName: 'Smith',
            commonAccessCardId: '1234567890',
          },
        };
        return Promise.resolve(true);
      }

      return Promise.resolve(false);
    },

    generateSignature() {
      return Promise.resolve(ok(Buffer.from('signature')));
    },

    getCertificate() {
      return Promise.resolve(Buffer.from('certificate'));
    },

    logOut() {
      currentStatus = {
        status: 'no_card',
      };
      return Promise.resolve();
    },
  };
}

export async function createApp(): Promise<MockAppContents> {
  const mockAuth = buildMockAuth();
  const workdir = dirSync().name;

  const app = await buildApp({
    auth: mockAuth,
    workspace: createWorkspace(workdir),
    logger: mockLogger(),
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
