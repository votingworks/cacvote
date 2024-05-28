import { safeParseJson } from '@votingworks/types';
import { Buffer } from 'buffer';
import { SignedObject, SignedObjectSchema, Uuid } from './types';

test('round-trip SignedObject', () => {
  const signedObject = new SignedObject(
    Uuid(),
    Uuid(),
    Buffer.from('payload'),
    Buffer.from('certificates'),
    Buffer.from('signature')
  );

  expect(
    safeParseJson(
      JSON.stringify(signedObject),
      SignedObjectSchema
    ).unsafeUnwrap()
  ).toEqual(signedObject);
});
