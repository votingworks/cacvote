import { QueryClient, useMutation } from '@tanstack/react-query';
import { QUERY_CLIENT_DEFAULT_OPTIONS } from '@votingworks/ui';

export function createQueryClient(): QueryClient {
  return new QueryClient({ defaultOptions: QUERY_CLIENT_DEFAULT_OPTIONS });
}

export const postScannedCode = {
  useMutation() {
    return useMutation(async (data: Uint8Array) => {
      const response = await fetch('/api/scanned-mailing-label-code', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/octet-stream',
        },
        body: data,
      });

      if (!response.ok) {
        throw new Error(response.statusText);
      }

      return await response.json();
    });
  },
} as const;
