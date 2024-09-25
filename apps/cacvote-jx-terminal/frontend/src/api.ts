import {
  QueryClient,
  useMutation,
  useQuery,
  useQueryClient,
} from '@tanstack/react-query';
import {
  BallotStyleId,
  ElectionDefinition,
  PrecinctId,
  safeParseJson,
} from '@votingworks/types';
import { QUERY_CLIENT_DEFAULT_OPTIONS } from '@votingworks/ui';
import { Buffer } from 'buffer';
import { useEffect } from 'react';
import { SessionData, SessionDataSchema } from './cacvote-server/session_data';
import { JurisdictionCode, Uuid } from './cacvote-server/types';

export function createQueryClient(): QueryClient {
  return new QueryClient({ defaultOptions: QUERY_CLIENT_DEFAULT_OPTIONS });
}

export const sessionData = {
  queryKey: () => ['sessionData'],

  /**
   * Query the session data. This should be used in components that need to
   * access the session data. It will automatically update when the session data
   * changes.
   */
  useQuery() {
    return useQuery<SessionData>(this.queryKey(), () => new Promise(() => {}), {
      staleTime: Infinity,
    });
  },

  /**
   * Call this at the root of the application, i.e. in the App component. It
   * should not be called anywhere else to prevent multiple instances of the
   * `EventSource` from running.
   */
  useRootQuery() {
    const queryClient = useQueryClient();

    useEffect(() => {
      const eventSource = new EventSource('/api/status-stream');

      eventSource.onmessage = (event) => {
        const newSessionData = safeParseJson(
          event.data,
          SessionDataSchema
        ).unsafeUnwrap();
        queryClient.setQueryData(sessionData.queryKey(), newSessionData);
      };

      return () => {
        eventSource.close();
      };
    }, [queryClient]);
  },
} as const;

export interface CreateElectionRequest {
  jurisdictionCode: JurisdictionCode;
  electionDefinition: ElectionDefinition;
  mailingAddress: string;
}

export interface CreateElectionResponse {
  id: Uuid;
}

export const createElection = {
  /**
   * Create a new election.
   */
  useMutation() {
    return useMutation(async (request: CreateElectionRequest) => {
      const response = await fetch('/api/elections', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          ...request,
          electionDefinition: Buffer.from(
            request.electionDefinition.electionData
          ).toString('base64'),
        }),
      });

      if (!response.ok) {
        throw new Error(`Failed to create election: ${response.statusText}`);
      }

      return (await response.json()) as CreateElectionResponse;
    });
  },
} as const;

export interface CreateRegistrationRequest {
  registrationRequestId: Uuid;
  electionId: Uuid;
  ballotStyleId: BallotStyleId;
  precinctId: PrecinctId;
}

export interface CreateRegistrationResponse {
  id: Uuid;
}

export const registerVoter = {
  useMutation() {
    return useMutation(async (request: CreateRegistrationRequest) => {
      const response = await fetch('/api/registrations', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(request),
      });

      if (!response.ok) {
        throw new Error(`Failed to register voter: ${response.statusText}`);
      }

      return (await response.json()) as CreateRegistrationResponse;
    });
  },
} as const;

export interface CreateEncryptedElectionTallyRequest {
  electionId: Uuid;
}

export interface CreateEncryptedElectionTallyResponse {
  id: Uuid;
}

export const generateEncryptedElectionTally = {
  useMutation() {
    return useMutation(async ({ electionId }: { electionId: Uuid }) => {
      const response = await fetch(
        `/api/elections/${electionId}/encrypted-tally`,
        { method: 'POST' }
      );

      if (!response.ok) {
        throw new Error(
          `Failed to generate encrypted election tally: ${response.statusText}`
        );
      }

      return (await response.json()) as CreateEncryptedElectionTallyResponse;
    });
  },
} as const;

export interface DecryptEncryptedElectionTallyRequest {
  electionId: Uuid;
}

export interface DecryptEncryptedElectionTallyResponse {
  id: Uuid;
}

export const decryptEncryptedElectionTally = {
  useMutation() {
    return useMutation(
      async ({ electionId }: DecryptEncryptedElectionTallyRequest) => {
        const response = await fetch(
          `/api/elections/${electionId}/decrypted-tally`,
          { method: 'POST' }
        );

        if (!response.ok) {
          throw new Error(
            `Failed to decrypt encrypted election tally: ${response.statusText}`
          );
        }

        return (await response.json()) as DecryptEncryptedElectionTallyResponse;
      }
    );
  },
} as const;

export interface ShuffleEncryptedBallotsRequest {
  electionId: Uuid;
  phases: number;
}

export interface ShuffleEncryptedBallotsResponse {
  id: Uuid;
}

export const shuffleEncryptedBallots = {
  useMutation() {
    return useMutation(
      async ({ electionId, phases }: ShuffleEncryptedBallotsRequest) => {
        const response = await fetch(
          `/api/elections/${electionId}/mixed-ballots`,
          {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ phases }),
          }
        );

        if (!response.ok) {
          throw new Error(
            `Failed to shuffle encrypted ballots: ${response.statusText}`
          );
        }

        return (await response.json()) as ShuffleEncryptedBallotsResponse;
      }
    );
  },
} as const;

export const authenticate = {
  useMutation() {
    return useMutation(async (pin: string) => {
      const response = await fetch('/api/authenticate', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ pin }),
      });

      if (!response.ok) {
        throw new Error(`Failed to authenticate: ${response.statusText}`);
      }
    });
  },
} as const;

/**
 * Gets the raw mailing label data for the given election. We don't parse it
 * here because we're just going to download it.
 */
export async function getScannedMailingLabelsRawByElection(
  electionId: Uuid
): Promise<Uint8Array> {
  const response = await fetch(
    `/api/elections/${electionId}/scanned-mailing-labels`
  );
  if (!response.ok) {
    throw new Error(
      `Failed to fetch scanned mailing labels: ${response.statusText}`
    );
  }

  const blob = await response.blob();
  return new Uint8Array(await blob.arrayBuffer());
}
