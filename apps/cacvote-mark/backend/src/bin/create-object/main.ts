import { cryptography } from '@votingworks/auth';
import { LogSource, Logger } from '@votingworks/logging';
import { Buffer } from 'buffer';
import { readFile } from 'fs/promises';
import { DateTime } from 'luxon';
import { join } from 'path';
import { Readable } from 'stream';
import {
  JurisdictionCodeSchema,
  Payload,
  RegistrationRequest,
  SignedObject,
  Uuid,
} from '../../cacvote-server/types';
import { Store } from '../../store';
import { resolveWorkspace } from '../../workspace';

const DEV_CERTS_PATH = join(__dirname, '../../../../../../libs/auth/certs/dev');
const PRIVATE_KEY_PATH = join(DEV_CERTS_PATH, 'vx-admin-private-key.pem');
const VX_ADMIN_CERT_AUTHORITY_CERT_PATH = join(
  DEV_CERTS_PATH,
  'vx-admin-cert-authority-cert.pem'
);

export async function main(): Promise<void> {
  const workspace = await resolveWorkspace(
    new Logger(LogSource.VxMarkBackend, () => Promise.resolve('system')),
    Store
  );

  const payload = Payload.RegistrationRequest(
    new RegistrationRequest(
      '0123456789',
      JurisdictionCodeSchema.parse('st.dev-jurisdiction'),
      'Jane',
      'Doe',
      DateTime.now()
    )
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
    Uuid(),
    Uuid(),
    payloadBuffer,
    certificatesPem,
    signature
  );

  // eslint-disable-next-line no-console
  console.log(await workspace.store.addObject(signedObject));
}
