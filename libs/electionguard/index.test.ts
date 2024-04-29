import { Buffer } from 'buffer';
import { assertDefined, iter } from '@votingworks/basics';
import { readElection } from '@votingworks/fs';
import { CVR, Election } from '@votingworks/types';
import { readFileSync } from 'fs';
import { join } from 'path';
import {
  convertVxCvrToEgPlaintextBallot,
  convertVxElectionToEgManifest,
  encryptEgPlaintextBallot,
  generateElectionConfig,
} from './index';

function readElectionFixture(name: string) {
  return readElection(join(__dirname, 'test/fixtures', name));
}

let famousNamesElection: Election;
let famousNamesCvr: CVR.CVR;
const { EG_CLASSPATH } = process.env;

const ZIP_MAGIC_BYTES = Buffer.from('PK');

function isZipBuffer(buf: Buffer) {
  return (
    Buffer.compare(
      buf.slice(0, ZIP_MAGIC_BYTES.byteLength),
      ZIP_MAGIC_BYTES
    ) === 0
  );
}

beforeAll(async () => {
  famousNamesElection = (
    await readElectionFixture('famous-names-election.json')
  ).unsafeUnwrap().election;

  famousNamesCvr = JSON.parse(
    readFileSync(
      join(__dirname, 'test/fixtures/famous-names-cast-vote-record.json'),
      'utf-8'
    )
  ) as CVR.CVR;
});

test('convert VX election to EG manifest', () => {
  const manifest = convertVxElectionToEgManifest(famousNamesElection) as {
    contests: unknown[];
    candidates: unknown[];
  };
  expect(manifest.contests.length).toEqual(famousNamesElection.contests.length);
  expect(manifest.candidates.length).toEqual(
    iter(famousNamesElection.contests)
      .filterMap((c) =>
        c.type === 'candidate' ? c.candidates.length : undefined
      )
      .sum()
  );
});

test('convert VX CVR to EG plaintext ballot', () => {
  const manifest = convertVxElectionToEgManifest(famousNamesElection);
  const egPlaintextBallot = convertVxCvrToEgPlaintextBallot(
    famousNamesElection,
    manifest,
    famousNamesCvr
  );
  expect(egPlaintextBallot).toMatchObject({
    ballot_id: famousNamesCvr.UniqueId,
    ballot_style: manifest.ballot_styles[0]?.object_id,
  });
});

(EG_CLASSPATH ? test : test.skip)('generate election config', () => {
  const manifest = convertVxElectionToEgManifest(famousNamesElection);
  const config = generateElectionConfig(assertDefined(EG_CLASSPATH), manifest);
  expect(config.privateMetadataBlob).toBeInstanceOf(Buffer);
  expect(isZipBuffer(config.privateMetadataBlob)).toBeTruthy();
  expect(config.publicMetadataBlob).toBeInstanceOf(Buffer);
  expect(isZipBuffer(config.publicMetadataBlob)).toBeTruthy();
});

(EG_CLASSPATH ? test : test.skip)('encrypt EG plaintext ballot', () => {
  const manifest = convertVxElectionToEgManifest(famousNamesElection);
  const config = generateElectionConfig(assertDefined(EG_CLASSPATH), manifest);
  const egPlaintextBallot = convertVxCvrToEgPlaintextBallot(
    famousNamesElection,
    manifest,
    famousNamesCvr
  );
  const encryptedBallot = encryptEgPlaintextBallot(
    assertDefined(EG_CLASSPATH),
    config.publicMetadataBlob,
    egPlaintextBallot,
    'test-device'
  );
  expect(encryptedBallot).toMatchObject({
    ballot_id: egPlaintextBallot['ballot_id'],
    ballot_style_id: egPlaintextBallot['ballot_style'],
  });
});
