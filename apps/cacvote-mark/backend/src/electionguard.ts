import { ELECTIONGUARD_CLASSPATH, VX_MACHINE_ID } from '@votingworks/backend';
import {
  Result,
  assert,
  assertDefined,
  err,
  iter,
  ok,
} from '@votingworks/basics';
import { CVR, Election } from '@votingworks/types';
import * as cp from 'child_process';
import { createReadStream, rmSync } from 'fs';
import { ensureDir } from 'fs-extra';
import { opendir, writeFile } from 'fs/promises';
import { join } from 'path';
import { dirSync } from 'tmp';
import { promisify } from 'util';
import ZipStream from 'zip-stream';
import { Uuid } from './cacvote-server/types';

const execFile = promisify(cp.execFile);

interface PlaintextBallot {
  ballot_id: string; // a unique ballot ID created by the external system
  ballot_style: string; // matches a Manifest.BallotStyle
  contests: PlaintextBallotContest[];
  sn?: number; // must be > 0
  errors?: string; // error messages from processing, eg when invalid
}

interface PlaintextBallotContest {
  contest_id: string; // matches ContestDescription.contestId
  sequence_order: number; // matches ContestDescription.sequenceOrder
  selections: Selection[];
  write_ins: string[];
}

interface PlaintextBallotSelection {
  selection_id: string; // matches SelectionDescription.selectionId
  sequence_order: number; // matches SelectionDescription.sequenceOrder
  vote: number;
}

enum ElectionType {
  Primary = 'primary',
  General = 'general',
}

interface GeopoliticalUnit {
  object_id: Uuid;
  name: string;
  type: GeopoliticalUnitType;
  contact_information?: ContactInformation[];
}

enum GeopoliticalUnitType {
  District = 'district',
}

interface Party {
  object_id: Uuid;
  name: string;
  abbreviation: string;
  color?: string;
  logo_uri?: string;
}

interface Contest {
  object_id: Uuid;
  sequence_order: number;
  electoral_district_id: Uuid;
  vote_variation: VoteVariation;
  number_elected: number;
  votes_allowed: number;
  name: string;
  ballot_selections: BallotSelection[];
  ballot_title?: string;
  ballot_subtitle?: string;
}

interface Candidate {
  object_id: Uuid;
  name: string;
  party_id?: Uuid;
  image_url?: string;
  is_write_in: boolean;
}

interface BallotSelection {
  object_id: Uuid;
  sequence_order: number;
  candidate_id: Uuid;
}

enum VoteVariation {
  OneOfM = 'one_of_m',
  NofM = 'n_of_m',
}

interface BallotStyle {
  object_id: Uuid;
  geopolitical_unit_ids: Uuid[];
  party_ids: Uuid[];
  image_uri?: string;
}

interface ContactInformation {
  name: string;
  address_line: string[];
  email?: string;
  phone?: string;
}

interface ElectionGuardManifest {
  election_scope_id: string;
  spec_version: string;
  type: ElectionType;
  start_date: string;
  end_date: string;
  geopolitical_units: GeopoliticalUnit[];
  parties: Party[];
  candidates: Candidate[];
  contests: Contest[];
  ballot_styles: BallotStyle[];
  name: string[];
  contact_information: ContactInformation;
}

export function convertCvrToPlaintextBallot(
  vxElection: Election,
  egManifest: ElectionGuardManifest,
  cvr: CVR.CVR
): PlaintextBallot {
  assert(
    vxElection.contests.length === egManifest.contests.length,
    'Both election representations must have the same number of contests'
  );

  for (const [i, [vxContest, egContest]] of iter(vxElection.contests)
    .zip(egManifest.contests)
    .enumerate()) {
    assert(
      egContest.sequence_order === i + 1,
      'Contests must be in sequence order'
    );

    if (vxContest.type === 'candidate') {
      assert(
        vxContest.candidates.length === egContest.ballot_selections.length,
        'Both election representations must have the same number of candidates'
      );
      assert(
        vxContest.seats === egContest.number_elected,
        'Both election representations must have the same number of seats'
      );

      for (const [j, egSelection] of iter(
        egContest.ballot_selections
      ).enumerate()) {
        assert(
          egSelection.sequence_order === j + 1,
          'Selections must be in sequence order'
        );
      }
    }
  }

  assert(cvr.CVRSnapshot.length === 1, 'CVR must have exactly one snapshot');
  const snapshot = assertDefined(cvr.CVRSnapshot[0]);

  assert(snapshot.Type === CVR.CVRType.Original, 'CVR must be original');
  snapshot.CVRContest.map<PlaintextBallotContest>((cvrContest) => {
    const vxContestIndex = vxElection.contests.findIndex(
      (c) => c.id === cvrContest.ContestId
    );
    assert(vxContestIndex >= 0, 'CVR contest must be in election');

    const vxContest = assertDefined(vxElection.contests[vxContestIndex]);
    const egContest = assertDefined(egManifest.contests[vxContestIndex]);

    const selections =
      cvrContest.CVRContestSelection.map<PlaintextBallotSelection>(
        (selection) => {
          assert(
            selection.SelectionPosition.length === 1,
            'Selection must be 1'
          );
          const position = assertDefined(selection.SelectionPosition[0]);
          assert(
            position.IsAllocable === CVR.AllocationStatus.Yes,
            'Selection must be allocable'
          );
          assert(
            position.HasIndication === CVR.IndicationStatus.Yes,
            'Selection must be indicated'
          );

          const vxSelectionIndex = vxContest.candidates.findIndex(
            (c) => c.id === selection.SelectionId
          );
          assert(vxSelectionIndex >= 0, 'CVR selection must be in contest');
          const vxSelection = assertDefined(
            vxContest.candidates[vxSelectionIndex]
          );
          const egSelection = assertDefined(
            egContest.ballot_selections[vxSelectionIndex]
          );

          return {
            selection_id: egSelection.object_id,
            sequence_order: egSelection.sequence_order,
            vote: selection.Status,
          };
        }
      );

    return {
      contest_id: egContest.object_id,
      sequence_order: egContest.sequence_order,
      selections,
      write_ins,
    };
  });
}

async function only<T>(
  iterable: AsyncIterable<T>
): Promise<Result<T, 'None' | 'TooMany'>> {
  const items = await iter(iterable).take(2).toArray();
  switch (items.length) {
    case 0:
      return err('None');

    case 1:
      return ok(items[0] as T);

    default:
      return err('TooMany');
  }
}

async function readSingleFileEntry(path: string): Promise<string> {
  return (
    await only(
      iter(await opendir(path))
        .filter((entry) => entry.isFile())
        .map((entry) => entry.name)
    )
  ).assertOk('could not get only file in directory');
}

export function encryptBallot(
  egManifest: ElectionGuardManifest,
  plaintextBallot: PlaintextBallot
): NodeJS.ReadableStream {
  const zip = new ZipStream();

  process.nextTick(async () => {
    try {
      const workingDir = dirSync().name;
      const inputBallotsDir = join(workingDir, 'input_ballots');
      const encryptedBallotsDir = join(workingDir, 'encrypted_ballots');
      const electionManifestPath = join(
        workingDir,
        'election_initialized.json'
      );

      await writeFile(
        electionManifestPath,
        JSON.stringify(egManifest, null, 2)
      );
      await ensureDir(inputBallotsDir);
      await ensureDir(encryptedBallotsDir);

      await writeFile(
        join(inputBallotsDir, `pballot-${plaintextBallot.ballotId}.json`),
        JSON.stringify(plaintextBallot, null, 2)
      );

      await execFile('java', [
        '-classpath',
        ELECTIONGUARD_CLASSPATH,
        'electionguard.cli.RunBatchEncryption',
        '-in',
        workingDir,
        '-ballots',
        inputBallotsDir,
        '-eballots',
        encryptedBallotsDir,
        '-device',
        VX_MACHINE_ID,
      ]);

      const encryptedBallotEntry =
        await readSingleFileEntry(encryptedBallotsDir);

      zip.on('close', () => {
        rmSync(workingDir, { recursive: true, force: true });
      });

      zip.entry(
        createReadStream(join(encryptedBallotsDir, encryptedBallotEntry)),
        { name: encryptedBallotEntry },
        (e) => {
          if (e) {
            zip.emit('error', e);
          }
        }
      );

      zip.finalize();
    } catch (e) {
      zip.emit('error', e);
    }
  });

  return zip;
}
