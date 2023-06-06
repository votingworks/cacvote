import { type Api } from '@votingworks/rave-mark-backend';
import React from 'react';
import * as grout from '@votingworks/grout';
import { QueryClient } from '@tanstack/react-query';
import { QUERY_CLIENT_DEFAULT_OPTIONS } from '@votingworks/ui';

export type ApiClient = grout.Client<Api>;

export function createApiClient(): ApiClient {
  return grout.createClient<Api>({ baseUrl: '/api' });
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
