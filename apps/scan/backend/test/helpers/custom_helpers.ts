import { Buffer } from 'buffer';
import {
  InsertedSmartCardAuthApi,
  buildMockInsertedSmartCardAuth,
} from '@votingworks/auth';
import { Result, deferred, ok } from '@votingworks/basics';
import {
  CustomScanner,
  ErrorCode,
  ImageColorDepthType,
  ImageFileFormat,
  ImageFromScanner,
  ScanSide,
  mocks,
} from '@votingworks/custom-scanner';
import {
  electionFamousNames2021Fixtures,
  sampleBallotImages,
} from '@votingworks/fixtures';
import * as grout from '@votingworks/grout';
import { getImageChannelCount } from '@votingworks/image-utils';
import { Logger, fakeLogger } from '@votingworks/logging';
import { SheetOf, mapSheet } from '@votingworks/types';
import { Application } from 'express';
import { Server } from 'http';
import { AddressInfo } from 'net';
import tmp from 'tmp';
import { MockUsb, createMockUsb } from '@votingworks/backend';
import { Api, buildApp } from '../../src/app';
import {
  PrecinctScannerInterpreter,
  createInterpreter,
} from '../../src/interpret';
import {
  Delays,
  createPrecinctScannerStateMachine,
} from '../../src/scanners/custom/state_machine';
import { Workspace, createWorkspace } from '../../src/util/workspace';
import { expectStatus, waitForStatus } from './shared_helpers';

export async function withApp(
  {
    delays = {},
    preconfiguredWorkspace,
  }: {
    delays?: Partial<Delays>;
    preconfiguredWorkspace?: Workspace;
  },
  fn: (context: {
    apiClient: grout.Client<Api>;
    app: Application;
    mockAuth: InsertedSmartCardAuthApi;
    mockScanner: jest.Mocked<CustomScanner>;
    workspace: Workspace;
    mockUsb: MockUsb;
    logger: Logger;
    interpreter: PrecinctScannerInterpreter;
    server: Server;
  }) => Promise<void>
): Promise<void> {
  const mockAuth = buildMockInsertedSmartCardAuth();
  const logger = fakeLogger();
  const workspace =
    preconfiguredWorkspace ?? (await createWorkspace(tmp.dirSync().name));
  const mockScanner = mocks.fakeCustomScanner();
  const deferredConnect = deferred<void>();
  async function createCustomClient(): Promise<
    Result<CustomScanner, ErrorCode>
  > {
    const connectResult = await mockScanner.connect();
    if (connectResult.isErr()) {
      return connectResult;
    }
    await deferredConnect.promise;
    return ok(mockScanner);
  }
  const interpreter = createInterpreter();
  const precinctScannerMachine = createPrecinctScannerStateMachine({
    createCustomClient,
    workspace,
    interpreter,
    logger,
    delays: {
      DELAY_RECONNECT: 100,
      DELAY_ACCEPTED_READY_FOR_NEXT_BALLOT: 100,
      DELAY_ACCEPTED_RESET_TO_NO_PAPER: 200,
      DELAY_PAPER_STATUS_POLLING_INTERVAL: 50,
      ...delays,
    },
  });
  const mockUsb = createMockUsb();
  const app = buildApp(
    mockAuth,
    precinctScannerMachine,
    interpreter,
    workspace,
    mockUsb.mock,
    logger
  );

  const server = app.listen();
  const { port } = server.address() as AddressInfo;
  const baseUrl = `http://localhost:${port}/api`;

  const apiClient = grout.createClient<Api>({ baseUrl });

  await expectStatus(apiClient, { state: 'connecting' });
  deferredConnect.resolve();
  await waitForStatus(apiClient, { state: 'no_paper' });

  try {
    await fn({
      apiClient,
      app,
      mockAuth,
      mockScanner,
      workspace,
      mockUsb,
      logger,
      interpreter,
      server,
    });
  } finally {
    const { promise, resolve, reject } = deferred<void>();
    server.close((error) => (error ? reject(error) : resolve()));
    await promise;
    workspace.reset();
  }
}

function customSheetOfImagesFromScannerFromBallotImageData(
  ballotImageData: SheetOf<ImageData>
): SheetOf<ImageFromScanner> {
  return mapSheet(ballotImageData, (imageData, side): ImageFromScanner => {
    const channelCount = getImageChannelCount(imageData);
    const imageDepth =
      channelCount === 1
        ? ImageColorDepthType.Grey8bpp
        : ImageColorDepthType.Color24bpp;

    return {
      scanSide: side === 'front' ? ScanSide.A : ScanSide.B,
      imageBuffer: Buffer.from(imageData.data),
      imageWidth: imageData.width,
      imageHeight: imageData.height,
      imageFormat: ImageFileFormat.Jpeg,
      imageDepth,
      imageResolution: Math.round(imageData.width / 8.5),
    };
  });
}

export const ballotImages = {
  completeHmpb: async () =>
    customSheetOfImagesFromScannerFromBallotImageData([
      await electionFamousNames2021Fixtures.handMarkedBallotCompletePage1.asImageData(),
      await electionFamousNames2021Fixtures.handMarkedBallotCompletePage2.asImageData(),
    ]),
  completeBmd: async () =>
    customSheetOfImagesFromScannerFromBallotImageData([
      await electionFamousNames2021Fixtures.machineMarkedBallotPage1.asImageData(),
      await electionFamousNames2021Fixtures.machineMarkedBallotPage2.asImageData(),
    ]),
  unmarkedHmpb: async () =>
    customSheetOfImagesFromScannerFromBallotImageData([
      await electionFamousNames2021Fixtures.handMarkedBallotUnmarkedPage1.asImageData(),
      await electionFamousNames2021Fixtures.handMarkedBallotUnmarkedPage2.asImageData(),
    ]),
  wrongElection: async () =>
    customSheetOfImagesFromScannerFromBallotImageData([
      // A BMD ballot front from a different election
      await sampleBallotImages.sampleBatch1Ballot1.asImageData(),
      // Blank BMD ballot back
      await electionFamousNames2021Fixtures.machineMarkedBallotPage2.asImageData(),
    ]),
  // The interpreter expects two different image files, so we use two
  // different blank page images
  blankSheet: async () =>
    customSheetOfImagesFromScannerFromBallotImageData([
      await sampleBallotImages.blankPage.asImageData(),
      // Blank BMD ballot back
      await electionFamousNames2021Fixtures.machineMarkedBallotPage2.asImageData(),
    ]),
} as const;