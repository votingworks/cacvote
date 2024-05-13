import { unsafeParse } from '@votingworks/types';
import { Buffer } from 'buffer';
import * as fc from 'fast-check';
import { UuidSchema } from './cacvote-server/types';
import { BallotVerificationPayload, SignedBuffer } from './verification';

function arbitrarySha256() {
  return fc.uint8Array({ minLength: 32, maxLength: 32 });
}

test(BallotVerificationPayload.name, () => {
  fc.assert(
    fc.property(
      fc.string({ minLength: 2, maxLength: 10 }), // machineId
      fc.string({ minLength: 10, maxLength: 10 }), // commonAccessCardId
      fc.uuid(),
      arbitrarySha256(),
      (machineId, commonAccessCardId, electionObjectId, signatureHash) => {
        const original = new BallotVerificationPayload(
          machineId,
          commonAccessCardId,
          unsafeParse(UuidSchema, electionObjectId),
          Buffer.from(signatureHash)
        );

        const encoded = original.encode();
        const decoded = BallotVerificationPayload.decode(encoded);
        expect(decoded).toEqual(original);
      }
    )
  );
});

test(SignedBuffer.name, () => {
  fc.assert(
    fc.property(fc.uint8Array(), arbitrarySha256(), (buffer, signature) => {
      const original = new SignedBuffer(
        Buffer.from(buffer),
        Buffer.from(signature)
      );

      const encoded = original.encode();
      const decoded = SignedBuffer.decode(encoded);
      expect(decoded).toEqual(original);
    })
  );
});
