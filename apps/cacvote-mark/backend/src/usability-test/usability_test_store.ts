import { ELECTIONGUARD_CLASSPATH } from '@votingworks/backend';
import { AsyncIteratorPlus, assert } from '@votingworks/basics';
import {
  convertVxElectionToEgManifest,
  generateElectionConfig,
} from '@votingworks/electionguard';
import { readElection } from '@votingworks/fs';
import { DateTime } from 'luxon';
import {
  JURISDICTION_CODE,
  createVerifiedObject,
} from '../cacvote-server/mock_object';
import {
  Election,
  Payload,
  Registration,
  RegistrationRequest,
  SignedObject,
  Uuid,
} from '../cacvote-server/types';
import {
  USABILITY_TEST_ELECTION_PATH,
  USABILITY_TEST_SKIP_REGISTRATION,
} from '../globals';
import { Store } from '../store';

export class UsabilityTestStore extends Store {
  override forEachElection(): AsyncIteratorPlus<{
    object: SignedObject;
    election: Election;
  }> {
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    const self = this;
    return super.forEachElection().chain(
      (async function* makeMockElection() {
        // If we get here, there is no election and we need to load the election
        // definition to create one.
        assert(
          typeof USABILITY_TEST_ELECTION_PATH === 'string',
          'USABILITY_TEST_ELECTION_PATH must be set when using UsabilityTestStore'
        );
        const electionDefinition = (
          await readElection(USABILITY_TEST_ELECTION_PATH)
        ).unsafeUnwrap();
        assert(
          typeof ELECTIONGUARD_CLASSPATH === 'string',
          'ELECTIONGUARD_CLASSPATH must be set'
        );

        const manifest = convertVxElectionToEgManifest(
          electionDefinition.election
        );
        const egElectionConfig = generateElectionConfig(
          ELECTIONGUARD_CLASSPATH,
          manifest
        );
        const election = new Election(
          JURISDICTION_CODE,
          electionDefinition,
          '123 Main St.\nAnytown, USA',
          egElectionConfig.publicMetadataBlob
        );
        const payload = Payload.Election(election);
        const object = await createVerifiedObject(payload);

        // Add the object to the store for use in future queries.
        (await self.addObject(object)).unsafeUnwrap();

        yield {
          object,
          election,
        };
      })()
    );
  }

  override forEachRegistrationRequest({
    commonAccessCardId,
  }: {
    commonAccessCardId: string;
  }): AsyncIteratorPlus<{
    object: SignedObject;
    registrationRequest: RegistrationRequest;
  }> {
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    const self = this;
    return super
      .forEachRegistrationRequest({
        commonAccessCardId,
      })
      .chain(
        (async function* makeMockRegistrationRequest() {
          if (!USABILITY_TEST_SKIP_REGISTRATION) {
            return;
          }

          // If we get here, there is no registration request for the given
          // common access card ID. We need to create a mock registration
          // request.
          const registrationRequest = new RegistrationRequest(
            commonAccessCardId,
            JURISDICTION_CODE,
            'Test',
            'Voter',
            DateTime.now()
          );
          const payload = Payload.RegistrationRequest(registrationRequest);
          const object = await createVerifiedObject(payload);

          // Add the object to the store for use in future queries.
          (await self.addObject(object)).unsafeUnwrap();

          yield {
            object,
            registrationRequest,
          };
        })()
      );
  }

  override forEachRegistration({
    commonAccessCardId,
    registrationRequestObjectId,
  }: {
    commonAccessCardId: string;
    registrationRequestObjectId?: Uuid;
  }): AsyncIteratorPlus<{ object: SignedObject; registration: Registration }> {
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    const self = this;
    return super
      .forEachRegistration({
        commonAccessCardId,
        registrationRequestObjectId,
      })
      .chain(
        (async function* makeMockRegistration() {
          if (!USABILITY_TEST_SKIP_REGISTRATION) {
            return;
          }

          // If we get here, there is no registration for the given common
          // access card ID. We need to create a mock registration.
          const registrationRequest = await self
            .forEachRegistrationRequest({ commonAccessCardId })
            .first();
          const election = await self.forEachElection().first();
          assert(
            registrationRequest,
            `No registration request found for CAC ID: ${commonAccessCardId}`
          );
          assert(election, 'No election found');

          const ballotStyle =
            election.election.getElectionDefinition().election.ballotStyles[0];
          assert(ballotStyle, 'No ballot style found');
          const precinctId = ballotStyle.precincts[0];
          assert(typeof precinctId === 'string', 'No precinct ID found');
          const registration = new Registration(
            commonAccessCardId,
            JURISDICTION_CODE,
            registrationRequest.object.getId(),
            election.object.getId(),
            ballotStyle.id,
            precinctId
          );
          const payload = Payload.Registration(registration);
          const object = await createVerifiedObject(payload);

          // Add the object to the store for use in future queries.
          (await self.addObject(object)).unsafeUnwrap();

          yield {
            object,
            registration,
          };
        })()
      );
  }
}
