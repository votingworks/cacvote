import { cryptography } from '@votingworks/auth';
import { Buffer } from 'buffer';
import { Readable } from 'stream';
import { join } from 'path';
import { readFile } from 'fs/promises';
import {
  ElectionObjectType,
  JurisdictionCodeSchema,
  Payload,
  SignedObject,
  Uuid,
} from './types';

export const JURISDICTION_CODE = JurisdictionCodeSchema.parse(
  'st.test-jurisdiction'
);

async function getSigningKeyCertificateAuthority(): Promise<Buffer> {
  return await readFile(
    join(
      __dirname,
      '../../../../../libs/auth/certs/dev/vx-admin-cert-authority-cert.pem'
    )
  );
}

function getSigningKeyPrivateKeyPath(): string {
  return join(
    __dirname,
    '../../../../../libs/auth/certs/dev/vx-admin-private-key.pem'
  );
}

export async function createVerifiedObject(
  payload: Payload,
  {
    electionId = payload.getObjectType() !== ElectionObjectType
      ? Uuid()
      : undefined,
  } = {}
): Promise<SignedObject> {
  const payloadBuffer = payload.toBuffer();
  const signature = await cryptography.signMessage({
    message: Readable.from(payloadBuffer),
    signingPrivateKey: {
      source: 'file',
      path: getSigningKeyPrivateKeyPath(),
    },
  });
  const object = new SignedObject(
    Uuid(),
    electionId,
    payloadBuffer,
    await getSigningKeyCertificateAuthority(),
    signature
  );
  (await object.verify()).unsafeUnwrap();
  return object;
}
