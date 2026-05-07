/**
 * Singleton CLI context — network resolution + Horizon client.
 *
 * Initialized once by the root commander `preAction` hook so every
 * command can call `useContext()` without re-parsing global flags or
 * re-instantiating Horizon clients.
 *
 * Honors NETWORK_PASSPHRASE / HORIZON_URL env overrides so operators
 * can point at private Horizon instances (futurenet, standalone,
 * mirrors) without code changes.
 *
 * Network name validation lives on the commander option (`.choices()`),
 * not here — by the time `initContext` runs the value is already known
 * to be 'testnet' or 'public'.
 */

import { Horizon } from '@stellar/stellar-sdk'
import { NETWORK_PRESETS } from './constants'

export type NetworkName = 'testnet' | 'public'

export interface ResolvedNetwork {
  name: NetworkName
  passphrase: string
  horizon: string
}

export interface Context {
  network: ResolvedNetwork
  server: Horizon.Server
}

let ctx: Context | undefined

/**
 * Initialize the singleton context. Called by the per-leaf `preAction`
 * hook attached inside `withNetwork()`, so by the time we get here the
 * `--network` option has already been validated by commander's
 * `.choices()` and is guaranteed to be 'testnet' or 'public'.
 */
export function initContext(opts: { network: NetworkName }): Context {
  const preset = NETWORK_PRESETS[opts.network]
  const network: ResolvedNetwork = {
    name: opts.network,
    passphrase: process.env.NETWORK_PASSPHRASE ?? preset.passphrase,
    horizon: process.env.HORIZON_URL ?? preset.horizon,
  }
  ctx = { network, server: new Horizon.Server(network.horizon) }
  return ctx
}

export function useContext(): Context {
  if (!ctx) {
    throw new Error('context not initialized — call initContext() before useContext()')
  }
  return ctx
}
