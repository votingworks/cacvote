import { QueryClient, useMutation } from '@tanstack/react-query';
import { Result, err, ok } from '@votingworks/basics';
import { QUERY_CLIENT_DEFAULT_OPTIONS } from '@votingworks/ui';

export function createQueryClient(): QueryClient {
  return new QueryClient({ defaultOptions: QUERY_CLIENT_DEFAULT_OPTIONS });
}

export const postScannedCode = {
  useMutation() {
    return useMutation(
      async (data: Uint8Array): Promise<Result<{ id: string }, string>> => {
        const response = await fetch('/api/scanned-mailing-label-code', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/octet-stream',
          },
          body: data,
        });

        if (!response.ok) {
          if (response.headers.get('Content-Type') === 'application/json') {
            const { error } = await response.json();
            return err(error);
          }

          throw new Error(response.statusText);
        }

        return ok(await response.json());
      }
    );
  },
} as const;
