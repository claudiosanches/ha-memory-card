import type { TFunction } from 'i18next';

/** An API failure. `code` is the server's error code, or `network`. */
export class ApiError extends Error {
  readonly code: string;
  readonly field?: string;

  constructor(code: string, field?: string) {
    super(code);
    this.code = code;
    this.field = field;
  }
}

type FetchResult<T> = { data?: T; error?: unknown; response: Response };

/** Resolves to the response data, or throws `ApiError`. */
export async function call<T>(request: Promise<FetchResult<T>>): Promise<T> {
  let result: FetchResult<T>;
  try {
    result = await request;
  } catch {
    throw new ApiError('network');
  }
  if (!result.response.ok) {
    const body = (result.error ?? {}) as { code?: unknown; field?: unknown };
    throw new ApiError(
      typeof body.code === 'string' ? body.code : 'internal',
      typeof body.field === 'string' ? body.field : undefined,
    );
  }
  return result.data as T;
}

/** Translated message for any error; unknown codes fall back to the generic one. */
export function errorMessage(t: TFunction, error: unknown): string {
  const code = error instanceof ApiError ? error.code : 'internal';
  const key = `errors.${code}` as const;
  return t(key as 'errors.internal', { defaultValue: t('errors.internal') });
}
