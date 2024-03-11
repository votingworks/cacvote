import { err, ok } from '@votingworks/basics';
import { Buffer } from 'buffer';
import { safeParseJson, unsafeParse } from '@votingworks/types';
import { DateTime } from 'luxon';
import { mockCacvoteServer } from '../../test/mock_cacvote_server';
import {
  JournalEntry,
  JurisdictionCodeSchema,
  SignedObject,
  UuidSchema,
} from './types';

const uuid = unsafeParse(UuidSchema, '123e4567-e89b-12d3-a456-426614174000');
const jurisdictionCode = unsafeParse(
  JurisdictionCodeSchema,
  'st.test-jurisdiction'
);

test('checkStatus success', async () => {
  const server = await mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case 'GET /api/status':
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end('{}');
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  expect(await server.client.checkStatus()).toEqual(ok());
  await server.stop();
});

test('checkStatus failure', async () => {
  const server = await mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case 'GET /api/status':
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end('{}');
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  expect(await server.client.checkStatus()).toEqual(
    err({ type: 'network', message: 'Internal Server Error' })
  );
  await server.stop();
});

test('createObject success', async () => {
  const server = await mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case 'POST /api/objects': {
        expect(req.headers['content-type']).toEqual('application/json');
        let body = '';
        req.on('readable', () => {
          const chunk = req.read();
          if (chunk) {
            body += chunk.toString();
          }
        });

        req.on('end', () => {
          const object = safeParseJson(body).unsafeUnwrap();
          expect(object).toEqual({
            id: uuid,
            payload: Buffer.of(1, 2, 3).toString('base64'),
            certificates: Buffer.of(4, 5, 6).toString('base64'),
            signature: Buffer.of(7, 8, 9).toString('base64'),
          });
          res.writeHead(200, { 'Content-Type': 'application/json' });
          res.end(uuid);
        });

        break;
      }

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  const object = new SignedObject(
    uuid,
    Buffer.of(1, 2, 3),
    Buffer.of(4, 5, 6),
    Buffer.of(7, 8, 9)
  );
  expect(await server.client.createObject(object)).toEqual(ok(uuid));
  await server.stop();
});

test('createObject network failure', async () => {
  const server = await mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case 'POST /api/objects':
        expect(req.headers['content-type']).toEqual('application/json');
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end('{}');
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  const object = new SignedObject(
    uuid,
    Buffer.of(1, 2, 3),
    Buffer.of(4, 5, 6),
    Buffer.of(7, 8, 9)
  );
  expect(await server.client.createObject(object)).toEqual(
    err({ type: 'network', message: 'Internal Server Error' })
  );
  await server.stop();
});

test('createObject schema failure', async () => {
  const server = await mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case 'POST /api/objects':
        expect(req.headers['content-type']).toEqual('application/json');
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end('not a uuid');
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  const object = new SignedObject(
    uuid,
    Buffer.of(1, 2, 3),
    Buffer.of(4, 5, 6),
    Buffer.of(7, 8, 9)
  );
  expect(await server.client.createObject(object)).toEqual(
    err(
      expect.objectContaining({ type: 'schema', message: expect.any(String) })
    )
  );
  await server.stop();
});

test('getObjectById success / no object', async () => {
  const server = await mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case `GET /api/objects/${uuid}`:
        res.writeHead(404, { 'Content-Type': 'application/json' });
        res.end('{}');
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  expect(await server.client.getObjectById(uuid)).toEqual(ok(undefined));
  await server.stop();
});

test('getObjectById success / with object', async () => {
  const server = await mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case `GET /api/objects/${uuid}`:
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(
          JSON.stringify(
            new SignedObject(
              uuid,
              Buffer.of(1, 2, 3),
              Buffer.of(4, 5, 6),
              Buffer.of(7, 8, 9)
            )
          )
        );
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  const object = new SignedObject(
    uuid,
    Buffer.of(1, 2, 3),
    Buffer.of(4, 5, 6),
    Buffer.of(7, 8, 9)
  );
  expect(await server.client.getObjectById(uuid)).toEqual(ok(object));
  await server.stop();
});

test('getObjectById network failure', async () => {
  const server = await mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case `GET /api/objects/${uuid}`:
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end('{}');
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  expect(await server.client.getObjectById(uuid)).toEqual(
    err({ type: 'network', message: 'Internal Server Error' })
  );
  await server.stop();
});

test('getObjectById schema failure', async () => {
  const server = await mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case `GET /api/objects/${uuid}`:
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end('not a signed object');
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  expect(await server.client.getObjectById(uuid)).toEqual(
    err(
      expect.objectContaining({ type: 'schema', message: expect.any(String) })
    )
  );
  await server.stop();
});

test('getJournalEntries success / no entries', async () => {
  const server = await mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case 'GET /api/journal-entries':
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end('[]');
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  expect(await server.client.getJournalEntries()).toEqual(ok([]));
  await server.stop();
});

test('getJournalEntries success / with entries', async () => {
  const createdAt = DateTime.now();
  const server = await mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case 'GET /api/journal-entries':
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(
          JSON.stringify([
            new JournalEntry(
              uuid,
              uuid,
              jurisdictionCode,
              'objectType',
              'action',
              createdAt
            ),
          ])
        );
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  expect(await server.client.getJournalEntries()).toEqual(
    ok([
      {
        id: uuid,
        objectId: uuid,
        jurisdiction: jurisdictionCode,
        objectType: 'objectType',
        action: 'action',
        createdAt,
      },
    ])
  );
  await server.stop();
});

test('getJournalEntries network failure', async () => {
  const server = await mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case 'GET /api/journal-entries':
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end('{}');
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  expect(await server.client.getJournalEntries()).toEqual(
    err({ type: 'network', message: 'Internal Server Error' })
  );
  await server.stop();
});

test('getJournalEntries schema failure', async () => {
  const createdAt = DateTime.now();
  const server = await mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case 'GET /api/journal-entries':
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(
          JSON.stringify([
            {
              id: uuid,
              objectId: uuid,
              jurisdiction: 'invalid jurisdiction',
              objectType: 'objectType',
              action: 'action',
              createdAt: createdAt.toISO(),
            },
          ])
        );
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  expect(await server.client.getJournalEntries()).toEqual(
    err(
      expect.objectContaining({ type: 'schema', message: expect.any(String) })
    )
  );
  await server.stop();
});
