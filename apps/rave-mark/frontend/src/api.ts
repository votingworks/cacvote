import { AuthStatus, type Api } from '@votingworks/rave-mark-backend';
import React, { useEffect } from 'react';
import * as grout from '@votingworks/grout';
import {
  QueryClient,
  QueryKey,
  useMutation,
  useQuery,
  useQueryClient,
} from '@tanstack/react-query';
import { QUERY_CLIENT_DEFAULT_OPTIONS } from '@votingworks/ui';

export type ApiClient = grout.Client<Api>;

const BASE_URL = '/api';

export function createApiClient(): ApiClient {
  return grout.createClient<Api>({ baseUrl: BASE_URL });
}

export const ApiClientContext = React.createContext<ApiClient | undefined>(
  undefined
);

export function useApiClient(): ApiClient {
  const apiClient = React.useContext(ApiClientContext);
  if (!apiClient) {
    throw new Error('ApiClientContext.Provider not found');
  }
  return apiClient;
}

export function createQueryClient(): QueryClient {
  return new QueryClient({ defaultOptions: QUERY_CLIENT_DEFAULT_OPTIONS });
}

export const getAuthStatus = {
  queryKey(): QueryKey {
    return ['getAuthStatus'];
  },
  useQuery() {
    const apiClient = useApiClient();
    const queryClient = useQueryClient();

    useEffect(() => {
      const eventSource = new EventSource(
        grout.methodUrl('watchAuthStatus', BASE_URL)
      );

      eventSource.addEventListener('message', (event) => {
        const authStatus = grout.deserialize(event.data) as AuthStatus;
        queryClient.setQueryData(getAuthStatus.queryKey(), authStatus);
      });

      eventSource.addEventListener('error', (event) => {
        // eslint-disable-next-line no-console
        console.error('Error from watchAuthStatus event source', event);
      });

      return () => {
        eventSource.close();
      };
    }, [queryClient]);

    return useQuery(this.queryKey(), () => apiClient.getAuthStatus(), {
      staleTime: Infinity,
    });
  },
} as const;

export const getVoterStatus = {
  queryKey(): QueryKey {
    return ['getVoterStatus'];
  },
  useQuery() {
    const apiClient = useApiClient();
    return useQuery(
      this.queryKey(),
      async () => (await apiClient.getVoterStatus()) ?? null,
      {
        staleTime: 0,
      }
    );
  },
} as const;

export const getElectionConfiguration = {
  queryKey(): QueryKey {
    return ['getElectionConfiguration'];
  },
  useQuery() {
    const apiClient = useApiClient();
    return useQuery(
      this.queryKey(),
      async () => (await apiClient.getElectionConfiguration()) ?? null,
      { staleTime: 0 }
    );
  },
} as const;

export const checkPin = {
  useMutation() {
    const apiClient = useApiClient();
    const queryClient = useQueryClient();
    return useMutation(apiClient.checkPin, {
      async onSuccess() {
        await queryClient.invalidateQueries();
      },
    });
  },
} as const;

export const createVoterRegistration = {
  useMutation() {
    const apiClient = useApiClient();
    const queryClient = useQueryClient();
    return useMutation(apiClient.createVoterRegistration, {
      async onSuccess() {
        await queryClient.invalidateQueries();
      },
    });
  },
} as const;

export const castBallot = {
  useMutation() {
    const apiClient = useApiClient();
    const queryClient = useQueryClient();
    return useMutation(apiClient.castBallot, {
      async onSuccess() {
        await queryClient.invalidateQueries();
      },
    });
  },
} as const;

export const sync = {
  useMutation() {
    const apiClient = useApiClient();
    const queryClient = useQueryClient();
    return useMutation(apiClient.sync, {
      async onSuccess() {
        await queryClient.invalidateQueries();
      },
    });
  },
} as const;

export const getServerSyncAttempts = {
  queryKey(): QueryKey {
    return ['getServerSyncAttempts'];
  },
  useQuery() {
    const apiClient = useApiClient();
    return useQuery(this.queryKey(), () => apiClient.getServerSyncAttempts(), {
      staleTime: 0,
      refetchInterval: 1000,
    });
  },
} as const;
