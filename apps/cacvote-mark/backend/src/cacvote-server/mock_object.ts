import { cryptography } from '@votingworks/auth';
import { Buffer } from 'buffer';
import { Readable } from 'stream';
import { join } from 'path';
import { readFile } from 'fs/promises';
import { assert } from '@votingworks/basics';
import {
  ElectionObjectType,
  JurisdictionCodeSchema,
  Payload,
  SignedObject,
  Uuid,
} from './types';
import { CAC_CA_CERTS, MACHINE_CA_CERT } from '../globals';

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
  assert(MACHINE_CA_CERT, 'MACHINE_CA_CERT is not set');
  assert(CAC_CA_CERTS, 'CAC_CA_CERTS is not set');
  (await object.verify(MACHINE_CA_CERT, CAC_CA_CERTS)).unsafeUnwrap();
  return object;
}
