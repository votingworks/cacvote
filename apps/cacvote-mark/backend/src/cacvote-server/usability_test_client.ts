import { ELECTIONGUARD_CLASSPATH } from '@votingworks/backend';
import {
  IteratorPlus,
  Optional,
  assert,
  assertDefined,
  asyncResultBlock,
  err,
  iter,
  ok,
} from '@votingworks/basics';
import {
  convertVxElectionToEgManifest,
  generateElectionConfig,
} from '@votingworks/electionguard';
import { ElectionDefinition } from '@votingworks/types';
import { DateTime } from 'luxon';
import { LogEventId, Logger } from '@votingworks/logging';
import { z } from 'zod';
import { ClientApi, ClientResult } from './client';
import { JURISDICTION_CODE, createVerifiedObject } from './mock_object';
import {
  CastBallotObjectType,
  Election,
  ElectionObjectType,
  JournalEntry,
  Payload,
  Registration,
  RegistrationObjectType,
  RegistrationRequestObjectType,
  SignedObject,
  Uuid,
} from './types';

export const AutomaticExpirationTypeSchema = z.enum([
  'castBallotOnly',
  'castBallotAndRegistration',
]);

export type AutomaticExpirationType = z.infer<
  typeof AutomaticExpirationTypeSchema
>;

/**
 * Provides a mock client API for usability testing, specifically:
 * - Automatically registers pending registration requests
 * - Automatically expires completed voting sessions
 */
export class UsabilityTestClient implements ClientApi {
  private readonly objects: Map<Uuid, SignedObject> = new Map();
  private readonly journalEntries: JournalEntry[] = [];
  private readonly logger: Logger;

  constructor({ logger }: { logger: Logger }) {
    this.logger = logger;
  }

  static async withElection(
    electionDefinition: ElectionDefinition,
    { logger }: { logger: Logger }
  ): Promise<ClientResult<UsabilityTestClient>> {
    assert(
      typeof ELECTIONGUARD_CLASSPATH === 'string',
      'ELECTIONGUARD_CLASSPATH must be set'
    );

    const manifest = convertVxElectionToEgManifest(electionDefinition.election);
    const egElectionConfig = generateElectionConfig(
      ELECTIONGUARD_CLASSPATH,
      manifest
    );
    const payload = Payload.Election(
      new Election(
        JURISDICTION_CODE,
        electionDefinition,
        '123 Main St.\nAnytown, USA',
        egElectionConfig.publicMetadataBlob
      )
    );
    const object = await createVerifiedObject(payload);

    const client = new UsabilityTestClient({ logger });
    return asyncResultBlock(async (bail) => {
      (await client.createObject(object)).okOrElse(bail);
      return ok(client);
    });
  }

  checkStatus(): Promise<ClientResult<void>> {
    return Promise.resolve(ok());
  }

  async createObject(signedObject: SignedObject): Promise<ClientResult<Uuid>> {
    const id = signedObject.getId();
    this.objects.set(id, signedObject);

    const journalEntryId = Uuid();
    const getPayloadResult = signedObject.getPayload();

    if (getPayloadResult.isErr()) {
      return err({
        type: 'schema',
        error: getPayloadResult.err(),
        message: 'Invalid payload',
      });
    }

    const payload = getPayloadResult.ok();
    const getJurisdictionCodeResult = await signedObject.getJurisdictionCode();

    if (getJurisdictionCodeResult.isErr()) {
      return err({
        type: 'schema',
        error: getJurisdictionCodeResult.err(),
        message: 'Invalid jurisdiction code',
      });
    }

    const jurisdictionCode = getJurisdictionCodeResult.ok();

    this.journalEntries.push(
      new JournalEntry(
        journalEntryId,
        id,
        signedObject.getElectionId(),
        jurisdictionCode,
        payload.getObjectType(),
        'create',
        DateTime.now()
      )
    );

    await this.autoRegisterPendingRequests();

    return ok(id);
  }

  getObjectById(uuid: Uuid): Promise<ClientResult<Optional<SignedObject>>> {
    const isDeleted = this.getJournalEntriesForObject(uuid).some(
      (entry) => entry.getAction() === 'delete'
    );

    return Promise.resolve(ok(isDeleted ? undefined : this.objects.get(uuid)));
  }

  async getJournalEntries(since?: Uuid): Promise<ClientResult<JournalEntry[]>> {
    const sinceIndex = since
      ? this.journalEntries.findIndex((entry) => entry.getId() === since)
      : -1;

    return Promise.resolve(ok(this.journalEntries.slice(sinceIndex + 1)));
  }

  private async autoRegisterPendingRequests(): Promise<void> {
    const getFirstElectionResult = iter(this.objects.values())
      .filterMap((object) => {
        const getPayloadResult =
          object.getPayloadAsObjectType(ElectionObjectType);

        if (getPayloadResult.isErr()) {
          return;
        }

        return [object, getPayloadResult.ok()] as const;
      })
      .first();

    if (!getFirstElectionResult) {
      return;
    }

    const [electionObject, electionPayload] = getFirstElectionResult;
    const electionDefinition = electionPayload
      .getData()
      .getElectionDefinition();
    const ballotStyle = assertDefined(
      electionDefinition.election.ballotStyles[0]
    );
    const precinctId = assertDefined(ballotStyle.precincts[0]);

    const nonPendingRegistrationRequestIds = iter(this.objects.values())
      .filterMap(
        (object) =>
          object
            .getPayloadAsObjectType(RegistrationObjectType)
            .ok()
            ?.getData()
            .getRegistrationRequestObjectId()
      )
      .toSet();

    const pendingRegistrationRequests = iter(this.objects.values()).filterMap(
      (object) => {
        const getPayloadResult = object.getPayloadAsObjectType(
          RegistrationRequestObjectType
        );

        if (
          getPayloadResult.isErr() ||
          nonPendingRegistrationRequestIds.has(object.getId())
        ) {
          return undefined;
        }

        return [object, getPayloadResult.ok()] as const;
      }
    );

    for (const [object, registrationRequest] of pendingRegistrationRequests) {
      const registrationRequestData = registrationRequest.getData();

      const registration = await createVerifiedObject(
        Payload.Registration(
          new Registration(
            registrationRequestData.getCommonAccessCardId(),
            registrationRequestData.getJurisdictionCode(),
            object.getId(),
            electionObject.getId(),
            ballotStyle.id,
            precinctId
          )
        ),
        { electionId: electionObject.getId() }
      );

      (await this.createObject(registration)).unsafeUnwrap();
    }
  }

  autoExpireCompletedVotingSessions({
    before,
    expire,
  }: {
    before: DateTime;
    expire: AutomaticExpirationType;
  }): void {
    const castBallotJournalEntries = iter(this.journalEntries).filter(
      (entry) =>
        entry.getObjectType() === CastBallotObjectType &&
        entry.getAction() === 'create' &&
        entry.getCreatedAt().diff(before).toMillis() < 0 &&
        !this.getJournalEntriesForObject(entry.getObjectId()).some(
          (e) => e.getAction() === 'delete'
        )
    );

    for (const castBallotJournalEntry of castBallotJournalEntries) {
      void this.logger.log(LogEventId.ClearingBallotData, 'system', {
        message: `Auto-expiring voting session ${castBallotJournalEntry.getId()}`,
      });
      const castBallotObject = assertDefined(
        this.objects.get(castBallotJournalEntry.getObjectId())
      );
      const castBallotPayload = castBallotObject
        .getPayloadAsObjectType(CastBallotObjectType)
        .unsafeUnwrap();
      const castBallotData = castBallotPayload.getData();
      this.deleteObject(castBallotObject.getId());

      if (expire === 'castBallotAndRegistration') {
        this.deleteObject(castBallotData.getRegistrationObjectId());
        this.deleteObject(castBallotData.getRegistrationRequestObjectId());
      }
    }
  }

  private getJournalEntriesForObject(id: Uuid): IteratorPlus<JournalEntry> {
    return iter(this.journalEntries).filter(
      (entry) => entry.getObjectId() === id
    );
  }

  private deleteObject(id: Uuid): Optional<Uuid> {
    const existingDeleteJournalEntry = this.getJournalEntriesForObject(id).find(
      (entry) => entry.getAction() === 'delete'
    );

    if (existingDeleteJournalEntry) {
      return existingDeleteJournalEntry.getId();
    }

    const object = this.objects.get(id);

    if (!object) {
      return;
    }

    const journalEntryId = Uuid();
    const payload = object.getPayload().unsafeUnwrap();

    const newDeleteJournalEntry = new JournalEntry(
      journalEntryId,
      id,
      object.getElectionId(),
      JURISDICTION_CODE,
      payload.getObjectType(),
      'delete',
      DateTime.now()
    );

    this.journalEntries.push(newDeleteJournalEntry);

    return journalEntryId;
  }
}
