import { Buffer } from 'buffer';
import { v4 } from 'uuid';
import { safeParseJson, unsafeParse } from '@votingworks/types';
import { SignedObject, SignedObjectSchema, UuidSchema } from './types';

test('round-trip SignedObject', () => {
  const signedObject = new SignedObject(
    unsafeParse(UuidSchema, v4()),
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
