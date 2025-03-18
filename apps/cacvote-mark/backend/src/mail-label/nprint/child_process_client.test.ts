import { iter } from '@votingworks/basics';
import { ChildProcessClient } from './child_process_client';
import { mockChildProcessClientTestServer } from '../../../test/child_process_client_test_server';

type Request =
  | {
      type: 'connect';
      replyTo: string;
    }
  | {
      type: 'disconnect';
      replyTo: string;
    };

type Response =
  | {
      type: 'connected';
      inReplyTo: string;
    }
  | {
      type: 'disconnected';
      inReplyTo: string;
    };

type Event =
  | {
      type: 'error';
    }
  | {
      type: 'event';
    };

test('accepts a child process and a logger', () => {
  const child = mockChildProcessClientTestServer([]);
  expect(
    new ChildProcessClient<Request, Response, Event>(child)
  ).toBeInstanceOf(ChildProcessClient);
});

test('sends and receives JSON messages over stdio', async () => {
  const child = mockChildProcessClientTestServer([
    {
      type: 'expect',
      line: '{"type":"connect","replyTo":"1"}',
    },
    {
      type: 'echo',
      line: '{"type": "connected", "inReplyTo": "1"}',
    },
  ]);
  const client = new ChildProcessClient<Request, Response, Event>(child);
  expect(
    await client.send({ type: 'connect', replyTo: '1' })
  ).toEqual<Response>({
    type: 'connected',
    inReplyTo: '1',
  });
});

test('handles events', async () => {
  const child = mockChildProcessClientTestServer([
    {
      type: 'echo',
      line: '{"type": "event"}',
    },
    {
      type: 'echo',
      line: '{"type": "error"}',
    },
  ]);
  const client = new ChildProcessClient<Request, Response, Event>(child);
  expect(await iter(client.events()).take(2).toArray()).toEqual([
    { type: 'event' },
    { type: 'error' },
  ]);
});

test('prohibits concurrent requests with the same replyTo ID', async () => {
  const child = mockChildProcessClientTestServer([]);
  const client = new ChildProcessClient<Request, Response, Event>(child);
  void client.send({ type: 'connect', replyTo: '1' });
  await expect(client.send({ type: 'connect', replyTo: '1' })).rejects.toThrow(
    'Duplicate replyTo: 1'
  );
});
