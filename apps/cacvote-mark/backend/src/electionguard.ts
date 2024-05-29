import { ELECTIONGUARD_CLASSPATH, VX_MACHINE_ID } from '@votingworks/backend';
import { assert, assertDefined } from '@votingworks/basics';
import {
  convertVxCvrToEgPlaintextBallot,
  encryptEgPlaintextBallot,
  extractManifestFromPublicMetadataBlob,
} from '@votingworks/electionguard';
import { CVR } from '@votingworks/types';
import { Buffer } from 'buffer';
import { CastBallot, Election, Payload, Uuid } from './cacvote-server/types';

/**
 * Create an encrypted ballot payload from a cast vote record. This uses
 * ElectionGuard to encrypt the ballot based on the election metadata and the
 * cast vote record.
 */
export function createEncryptedBallotPayload(
  commonAccessCardId: string,
  electionPayload: Payload<Election>,
  registrationRequestObjectId: Uuid,
  registrationObjectId: Uuid,
  electionObjectId: Uuid,
  castVoteRecord: CVR.CVR,
  serialNumber: number
): Payload<CastBallot> {
  assert(
    Number.isSafeInteger(serialNumber),
    'serialNumber must be a safe integer'
  );
  assert(serialNumber >= 0, 'serialNumber must be non-negative');

  const election = electionPayload.getData();
  const electionMetadataBlob = election.getElectionguardElectionMetadataBlob();
  const manifest = extractManifestFromPublicMetadataBlob(electionMetadataBlob);
  const plaintextBallot = convertVxCvrToEgPlaintextBallot(
    election.getElectionDefinition().election,
    serialNumber,
    manifest,
    castVoteRecord
  );
  const encryptedBallot = encryptEgPlaintextBallot(
    assertDefined(ELECTIONGUARD_CLASSPATH),
    electionMetadataBlob,
    plaintextBallot,
    VX_MACHINE_ID
  );

  return Payload.CastBallot(
    new CastBallot(
      commonAccessCardId,
      election.getJurisdictionCode(),
      registrationRequestObjectId,
      registrationObjectId,
      electionObjectId,
      Buffer.from(JSON.stringify(encryptedBallot, null, 2))
    )
  );
}
