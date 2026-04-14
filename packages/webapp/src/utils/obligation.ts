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

function normalizeMultiplyStrategyAddress(address: string): string {
  return address.trim()
}

export function buildMultiplyStrategyKey(params: {
  borrowTokenAddress: string
  depositTokenAddress: string
}): string {
  return `${normalizeMultiplyStrategyAddress(params.borrowTokenAddress)}/${normalizeMultiplyStrategyAddress(params.depositTokenAddress)}`
}

export function buildMultiplyPairKey(depositPoolAddress: string, borrowPoolAddress: string): string {
  return `${depositPoolAddress}:${borrowPoolAddress}`
}

export async function buildMultiplyObligationSeed(params: {
  borrowTokenAddress: string
  depositTokenAddress: string
}): Promise<Buffer> {
  const encoder = new TextEncoder()
  const data = encoder.encode(buildMultiplyStrategyKey(params))
  const hash = await crypto.subtle.digest('SHA-256', data)

  return Buffer.from(hash)
}

export async function buildMultiplyObligationKey(params: {
  publicKey: string
  borrowTokenAddress: string
  depositTokenAddress: string
}): Promise<ObligationKey> {
  return {
    user: params.publicKey,
    seed: await buildMultiplyObligationSeed(params),
  }
}
