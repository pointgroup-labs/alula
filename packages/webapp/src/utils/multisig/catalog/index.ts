/**
 * Catalog registry — maps function_id to its FunctionDef.
 *
 * Each privileged function is registered as one file under
 * catalog/{role}/. To add a function: create the file, import + register it
 * here. The pages and lib are catalog-agnostic; they look entries up by id.
 *
 * Phase 1 ships the Upgrade catalog only. Program (Plan 3) and Ops (Plan 4)
 * register their entries in the same map.
 */

import type { FunctionDef, MultisigRole } from '../types'

import { applyInManagerUpgrade } from './upgrade/apply-in-manager-upgrade'
import { applyInMarketUpgrade } from './upgrade/apply-in-market-upgrade'
import { cancelInManagerUpgrade } from './upgrade/cancel-in-manager-upgrade'
import { cancelInMarketUpgrade } from './upgrade/cancel-in-market-upgrade'
import { queueInManagerUpgrade } from './upgrade/queue-in-manager-upgrade'
import { queueInMarketUpgrade } from './upgrade/queue-in-market-upgrade'

const ENTRIES: FunctionDef<any, any>[] = [
  queueInMarketUpgrade,
  applyInMarketUpgrade,
  cancelInMarketUpgrade,
  queueInManagerUpgrade,
  applyInManagerUpgrade,
  cancelInManagerUpgrade,
]

const BY_ID = new Map<string, FunctionDef<any, any>>(ENTRIES.map(e => [e.id, e]))

export function getFunctionDef(id: string): FunctionDef<any, any> | undefined {
  return BY_ID.get(id)
}

export function listFunctionsByRole(role: MultisigRole): FunctionDef<any, any>[] {
  return ENTRIES.filter(e => e.multisig === role)
}

export function listAllFunctions(): FunctionDef<any, any>[] {
  return [...ENTRIES]
}
