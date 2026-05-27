export const errorMap: Record<string, string> = {
  'Network Error': 'Network Error. Go to settings to change Soroban network.',
  'Failed to fetch': 'Server is unavailable.',
  'User rejected the request': 'Transaction was rejected.',
}

export function parseErrorMessage(
  error: string | Error | ErrorWithMessage | null | undefined,
  defaultMessage?: string,
): string {
  if (!error && defaultMessage) { return defaultMessage }

  const message = typeof error === 'object' && error !== null && 'message' in error
    ? String(error.message)
    : String(error)

  return errorMap[message] || defaultMessage || message
}

type ErrorWithMessage = {
  message?: string
}
