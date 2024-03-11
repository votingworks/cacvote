import { Buffer } from 'buffer';
import { readFile } from 'fs/promises';
import { cryptography } from '@votingworks/auth';
import { v4 } from 'uuid';
import { Readable } from 'stream';
import { unsafeParse } from '@votingworks/types';
import { join } from 'path';
import { Payload, SignedObject, UuidSchema } from '../../cacvote-server/types';
import { resolveWorkspace } from '../../workspace';

const DEV_CERTS_PATH = join(__dirname, '../../../../../../libs/auth/certs/dev');
const PRIVATE_KEY_PATH = join(DEV_CERTS_PATH, 'vx-admin-private-key.pem');
const VX_ADMIN_CERT_AUTHORITY_CERT_PATH = join(
  DEV_CERTS_PATH,
  'vx-admin-cert-authority-cert.pem'
);

export async function main(): Promise<void> {
  const workspace = await resolveWorkspace();

  interface TestObject {
    name: string;
    description: string;
    value: number;
  }

  const object: TestObject = {
    name: 'Test Object',
    description: 'This is a test object',
    value: 42,
  };

  const payload = new Payload(
    'TestObject',
    Buffer.from(JSON.stringify(object))
  );

  const certificatesPem = await readFile(VX_ADMIN_CERT_AUTHORITY_CERT_PATH);
  const payloadBuffer = Buffer.from(JSON.stringify(payload));
  const signature = await cryptography.signMessage({
    message: Readable.from(payloadBuffer),
    signingPrivateKey: {
      source: 'file',
      path: PRIVATE_KEY_PATH,
    },
  });
  const signedObject = new SignedObject(
    unsafeParse(UuidSchema, v4()),
    payloadBuffer,
    certificatesPem,
    signature
  );

  console.log(await workspace.store.addObject(signedObject));
}
