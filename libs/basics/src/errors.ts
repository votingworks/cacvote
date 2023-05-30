/**
 * Extracts the error message from an error in a type-safe way
 */
export function extractErrorMessage(error: unknown): string {
  return error instanceof Error
    ? error.message
    : typeof error === 'object' &&
      error !== null &&
      'message' in error &&
      typeof (error as { message?: unknown }).message === 'string'
    ? (error as { message: string }).message
    : error &&
      typeof error === 'object' &&
      'toString' in (error as { toString(): string })
    ? (error as { toString(): string }).toString()
    : String(error);
}
