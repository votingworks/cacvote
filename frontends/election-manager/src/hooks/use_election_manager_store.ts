import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  LogDispositionStandardTypes,
  LogEventId,
  Logger,
  LoggingUserRole,
} from '@votingworks/logging';
import {
  ElectionDefinition,
  FullElectionExternalTally,
  Iso8601Timestamp,
} from '@votingworks/types';
import { typedAs } from '@votingworks/utils';
import { useCallback, useMemo, useRef } from 'react';
import { PrintedBallot } from '../config/types';
import { ElectionManagerStoreBackend } from '../lib/backends/types';
import { CastVoteRecordFiles } from '../utils/cast_vote_record_files';

export interface ElectionManagerStore {
  /**
   * The currently configured election definition.
   */
  readonly electionDefinition?: ElectionDefinition;

  /**
   * The time at which the election definition was configured.
   */
  readonly configuredAt?: Iso8601Timestamp;

  /**
   * The current set of loaded cast vote record files.
   */
  readonly castVoteRecordFiles: CastVoteRecordFiles;

  /**
   * Whether the results have been marked as official.
   */
  readonly isOfficialResults: boolean;

  /**
   * Information about the ballots that have been printed.
   */
  readonly printedBallots: readonly PrintedBallot[];

  /**
   * Tallies from external sources, e.g. SEMS or manually entered tallies.
   */
  readonly fullElectionExternalTallies: readonly FullElectionExternalTally[];

  /**
   * Resets all stored data, including the election definition and CVRs.
   */
  reset(): Promise<void>;

  /**
   * Configures with a new election definition after resetting.
   *
   * @param newElectionData election definition as JSON string
   */
  configure(newElectionData: string): Promise<ElectionDefinition>;

  /**
   * Overwrites the existing cast vote record files with the given ones.
   */
  setCastVoteRecordFiles(
    newCastVoteRecordFiles: CastVoteRecordFiles
  ): Promise<void>;

  /**
   * Resets all cast vote record files.
   */
  clearCastVoteRecordFiles(): Promise<void>;

  /**
   * Adds an external tally to the list.
   */
  addFullElectionExternalTally(
    newFullElectionExternalTally: FullElectionExternalTally
  ): Promise<void>;

  /**
   * Replaces all external tallies with the given ones.
   */
  setFullElectionExternalTallies(
    newFullElectionExternalTallies: readonly FullElectionExternalTally[]
  ): Promise<void>;

  /**
   * Marks the results as official. No more tallies can be added after this.
   */
  markResultsOfficial(): Promise<void>;

  /**
   * Adds a new printed ballot to the list.
   */
  addPrintedBallot(printedBallot: PrintedBallot): Promise<void>;

  /**
   * Sets the current user's role, i.e. the person taking action.
   */
  setCurrentUserRole(newCurrentUserRole: LoggingUserRole): void;
}

interface Props {
  logger: Logger;
  backend: ElectionManagerStoreBackend;
}

export const electionDefinitionStorageKey = 'electionDefinition';
export const cvrsStorageKey = 'cvrFiles';
export const isOfficialResultsKey = 'isOfficialResults';
export const printedBallotsStorageKey = 'printedBallots';
export const configuredAtStorageKey = 'configuredAt';
export const externalVoteTalliesFileStorageKey = 'externalVoteTallies';

/**
 * Manages the stored data for VxAdmin.
 */
export function useElectionManagerStore({
  logger,
  backend,
}: Props): ElectionManagerStore {
  const queryClient = useQueryClient();
  const currentUserRoleRef = useRef<LoggingUserRole>('unknown');

  const getPrintedBallotsQuery = useQuery<PrintedBallot[]>(
    [printedBallotsStorageKey],
    async () => {
      return (await backend.loadPrintedBallots()) ?? [];
    }
  );
  const printedBallots = getPrintedBallotsQuery.data;

  const loadElectionDefinition = useCallback(async (): Promise<
    ElectionDefinition | undefined
  > => {
    const electionRecord =
      await backend.loadElectionDefinitionAndConfiguredAt();
    return electionRecord?.electionDefinition;
  }, [backend]);

  const getElectionDefinitionQuery = useQuery<ElectionDefinition | null>(
    [electionDefinitionStorageKey],
    async () => {
      // Return `null` if there is no election definition because `react-query`
      // does not allow returning `undefined` for query results.
      return (await loadElectionDefinition()) ?? null;
    }
  );
  const electionDefinition = getElectionDefinitionQuery.data ?? undefined;

  const loadConfiguredAt = useCallback(async (): Promise<
    string | undefined
  > => {
    const electionRecord =
      await backend.loadElectionDefinitionAndConfiguredAt();
    return electionRecord?.configuredAt;
  }, [backend]);

  const getConfiguredAtQuery = useQuery<string | null>(
    [configuredAtStorageKey],
    async () => {
      // Return `null` if there is no timestamp because `react-query` does not
      // allow returning `undefined` for query results.
      return (await loadConfiguredAt()) ?? null;
    }
  );
  const configuredAt = getConfiguredAtQuery.data ?? undefined;

  const getCastVoteRecordFilesQuery = useQuery<CastVoteRecordFiles>(
    [cvrsStorageKey],
    async () => {
      return (
        (await backend.loadCastVoteRecordFiles()) ?? CastVoteRecordFiles.empty
      );
    }
  );
  const castVoteRecordFiles = getCastVoteRecordFilesQuery.data;

  const getExternalElectionTalliesQuery = useQuery<FullElectionExternalTally[]>(
    [externalVoteTalliesFileStorageKey],
    async () => {
      return (await backend.loadFullElectionExternalTallies()) ?? [];
    }
  );
  const fullElectionExternalTallies = getExternalElectionTalliesQuery.data;

  const getIsOfficialResultsQuery = useQuery<boolean>(
    [isOfficialResultsKey],
    async () => {
      return (await backend.loadIsOfficialResults()) ?? false;
    }
  );
  const isOfficialResults = getIsOfficialResultsQuery.data;

  const reset = useCallback(async () => {
    await backend.reset();
    if (electionDefinition) {
      await logger.log(
        LogEventId.ElectionUnconfigured,
        currentUserRoleRef.current,
        {
          disposition: LogDispositionStandardTypes.Success,
          previousElectionHash: electionDefinition.electionHash,
        }
      );
    }
    await queryClient.invalidateQueries([electionDefinitionStorageKey]);
    await queryClient.invalidateQueries([cvrsStorageKey]);
    await queryClient.invalidateQueries([externalVoteTalliesFileStorageKey]);
    await queryClient.invalidateQueries([isOfficialResultsKey]);
    await queryClient.invalidateQueries([printedBallotsStorageKey]);
  }, [backend, electionDefinition, logger, queryClient]);

  const configure = useCallback(
    async (newElectionData: string): Promise<ElectionDefinition> => {
      await reset();
      const newElectionDefinition = await backend.configure(newElectionData);
      await logger.log(
        LogEventId.ElectionConfigured,
        currentUserRoleRef.current,
        {
          disposition: LogDispositionStandardTypes.Success,
          newElectionHash: newElectionDefinition.electionHash,
        }
      );
      await queryClient.invalidateQueries([electionDefinitionStorageKey]);
      await queryClient.invalidateQueries([configuredAtStorageKey]);
      return newElectionDefinition;
    },
    [backend, logger, queryClient, reset]
  );

  const setCastVoteRecordFilesMutation = useMutation(
    async (newCastVoteRecordFiles: CastVoteRecordFiles) => {
      await backend.setCastVoteRecordFiles(newCastVoteRecordFiles);
    },
    {
      onSuccess(data, newCastVoteRecordFiles) {
        void queryClient.invalidateQueries([cvrsStorageKey]);
        if (newCastVoteRecordFiles === CastVoteRecordFiles.empty) {
          void queryClient.invalidateQueries([isOfficialResultsKey]);
        }
      },
    }
  );

  const setCastVoteRecordFiles = useCallback(
    async (newCastVoteRecordFiles: CastVoteRecordFiles) => {
      await setCastVoteRecordFilesMutation.mutateAsync(newCastVoteRecordFiles);
    },
    [setCastVoteRecordFilesMutation]
  );

  const clearCastVoteRecordFiles = useCallback(async () => {
    await setCastVoteRecordFiles(CastVoteRecordFiles.empty);
  }, [setCastVoteRecordFiles]);

  const markResultsOfficialMutation = useMutation(
    async () => {
      await backend.markResultsOfficial();
      await logger.log(
        LogEventId.MarkedTallyResultsOfficial,
        currentUserRoleRef.current,
        {
          message:
            'User has marked the tally results as official, no more Cvr files can be loaded.',
          disposition: 'success',
        }
      );
    },
    {
      onSuccess() {
        void queryClient.invalidateQueries([isOfficialResultsKey]);
      },
    }
  );

  const markResultsOfficial = useCallback(async () => {
    await markResultsOfficialMutation.mutateAsync();
  }, [markResultsOfficialMutation]);

  const addPrintedBallotMutation = useMutation(
    async (newPrintedBallot: PrintedBallot) => {
      await backend.addPrintedBallot(newPrintedBallot);
    },
    {
      onSuccess() {
        void queryClient.invalidateQueries([printedBallotsStorageKey]);
      },
    }
  );

  const addPrintedBallot = useCallback(
    async (newPrintedBallot: PrintedBallot) => {
      await addPrintedBallotMutation.mutateAsync(newPrintedBallot);
    },
    [addPrintedBallotMutation]
  );

  const addFullElectionExternalTallyMutation = useMutation(
    async (newFullElectionExternalTally: FullElectionExternalTally) => {
      await backend.addFullElectionExternalTally(newFullElectionExternalTally);
    },
    {
      onSuccess() {
        void queryClient.invalidateQueries([externalVoteTalliesFileStorageKey]);
      },
    }
  );

  const addFullElectionExternalTally = useCallback(
    async (newFullElectionExternalTally: FullElectionExternalTally) => {
      await addFullElectionExternalTallyMutation.mutateAsync(
        newFullElectionExternalTally
      );
    },
    [addFullElectionExternalTallyMutation]
  );

  const setFullElectionExternalTalliesMutation = useMutation(
    async (
      newFullElectionExternalTallies: readonly FullElectionExternalTally[]
    ) => {
      await backend.setFullElectionExternalTallies(
        newFullElectionExternalTallies
      );
    },
    {
      onSuccess() {
        void queryClient.invalidateQueries([externalVoteTalliesFileStorageKey]);
      },
    }
  );

  const setFullElectionExternalTallies = useCallback(
    async (
      newFullElectionExternalTallies: readonly FullElectionExternalTally[]
    ) => {
      await setFullElectionExternalTalliesMutation.mutateAsync(
        newFullElectionExternalTallies
      );
    },
    [setFullElectionExternalTalliesMutation]
  );

  const setCurrentUserRole = useCallback((newCurrentUserRole) => {
    currentUserRoleRef.current = newCurrentUserRole;
  }, []);

  return useMemo(
    () =>
      typedAs<ElectionManagerStore>({
        castVoteRecordFiles: castVoteRecordFiles ?? CastVoteRecordFiles.empty,
        configuredAt,
        electionDefinition,
        fullElectionExternalTallies: fullElectionExternalTallies ?? [],
        isOfficialResults: isOfficialResults ?? false,
        printedBallots: printedBallots ?? [],

        addFullElectionExternalTally,
        addPrintedBallot,
        clearCastVoteRecordFiles,
        configure,
        markResultsOfficial,
        reset,
        setCastVoteRecordFiles,
        setCurrentUserRole,
        setFullElectionExternalTallies,
      }),
    [
      addFullElectionExternalTally,
      addPrintedBallot,
      castVoteRecordFiles,
      clearCastVoteRecordFiles,
      configure,
      configuredAt,
      electionDefinition,
      fullElectionExternalTallies,
      isOfficialResults,
      markResultsOfficial,
      printedBallots,
      reset,
      setCastVoteRecordFiles,
      setCurrentUserRole,
      setFullElectionExternalTallies,
    ]
  );
}