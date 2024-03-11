import {
  IncomingMessage,
  RequestListener,
  Server,
  ServerResponse,
  createServer,
} from 'http';
import { AddressInfo } from 'net';
import { Client } from '../src/cacvote-server/client';

export interface MockCacvoteServer {
  inner: Server;
  client: Client;
  stop(): Promise<void>;
}

export function mockCacvoteServer<
  Request extends typeof IncomingMessage,
  Response extends typeof ServerResponse,
>(handler: RequestListener<Request, Response>): MockCacvoteServer {
  const server = createServer(handler).listen();
  const address = server.address() as AddressInfo;
  const url = new URL(`http://[${address.address}]:${address.port}`);
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
