import type { ObligationKey } from '@alula/market-sdk'
import { Buffer } from 'node:buffer'

export async function buildObligationSeed(params: {
  user: string
  market: string
  pool: string
}): Promise<Buffer> {
  const encoder = new TextEncoder()

  const data = encoder.encode(
    `${params.user}-${params.market}-${params.pool}`,
  )

  const hash = await crypto.subtle.digest('SHA-256', data)

  return Buffer.from(hash)
}

export function buildObligationKey(params: {
  pablicKey: string
}): ObligationKey {
  return {
    user: params.pablicKey,
    seed: undefined,
  }
}
