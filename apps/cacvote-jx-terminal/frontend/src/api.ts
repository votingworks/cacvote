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
