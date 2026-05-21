import { VALID_PAIRS } from '@alula/client-sdk/src/constants'

export function normalizePair(a: string, b: string) {
  return [a, b].toSorted().join('-')
}

export function isValidPair(from: string, to: string) {
  return VALID_PAIRS.has(normalizePair(from, to))
}
