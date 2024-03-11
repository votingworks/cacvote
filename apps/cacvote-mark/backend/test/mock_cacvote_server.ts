import {
  IncomingMessage,
  RequestListener,
  Server,
  ServerResponse,
  createServer,
} from 'http';
import { AddressInfo } from 'net';
import { deferred } from '@votingworks/basics';
import { Client } from '../src/cacvote-server/client';

export interface MockCacvoteServer {
  inner: Server;
  client: Client;
  stop(): Promise<void>;
}

export async function mockCacvoteServer<
  Request extends typeof IncomingMessage,
  Response extends typeof ServerResponse,
>(handler: RequestListener<Request, Response>): Promise<MockCacvoteServer> {
  const listening = deferred<void>();
  const server = createServer(handler).listen(
    { host: '127.0.0.1', port: 0 },
    listening.resolve
  );
  await listening.promise;
  const address = server.address() as AddressInfo;
  const url = new URL(`http://127.0.0.1:${address.port}`);
  const client = new Client(url);
  return {
    inner: server,
    client,
    async stop() {
      await new Promise((resolve) => {
        server.close(resolve);
      });
    },
  };
}
