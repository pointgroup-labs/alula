/**
 * Cloudflare Worker relay client.
 *
 * Posts a sig payload to the relay and reads back the current sig list for a
 * proposal. The relay is convenience only — every sig is re-validated
 * client-side, and a network failure is non-fatal (manual paste fallback).
 *
 * Spec §7.
 */

import type { SigPayload } from './types'
import { extractSigPayloads, isWellFormedSigPayload, parseSigPayload, serializeSigPayload } from './url'

export interface RelayConfig {
  /** Base URL, e.g. "https://app.alula.fi/api/multisig" */
  baseUrl: string
  /** Per-request timeout in ms */
  timeoutMs?: number
}

export interface RelayResult<T> {
  ok: boolean
  data?: T
  error?: string
}

export async function postSig(cfg: RelayConfig, sig: SigPayload): Promise<RelayResult<void>> {
  const body = serializeSigPayload(sig)
  try {
    const res = await fetchWithTimeout(
      `${cfg.baseUrl}/sigs/${encodeURIComponent(sig.proposal_hash)}`,
      { method: 'POST', body, headers: { 'content-type': 'text/plain' } },
      cfg.timeoutMs ?? 5000,
    )
    if (!res.ok) return { ok: false, error: `relay returned ${res.status}` }
    return { ok: true }
  }
  catch (e) {
    return { ok: false, error: e instanceof Error ? e.message : 'relay unreachable' }
  }
}

export async function fetchSigs(cfg: RelayConfig, proposalHash: string): Promise<RelayResult<SigPayload[]>> {
  try {
    const res = await fetchWithTimeout(
      `${cfg.baseUrl}/sigs/${encodeURIComponent(proposalHash)}`,
      { method: 'GET' },
      cfg.timeoutMs ?? 5000,
    )
    if (!res.ok) return { ok: false, error: `relay returned ${res.status}` }
    const text = await res.text()
    const sigs = extractSigPayloads(text).filter(s => s.proposal_hash === proposalHash)
    return { ok: true, data: sigs }
  }
  catch (e) {
    return { ok: false, error: e instanceof Error ? e.message : 'relay unreachable' }
  }
}

async function fetchWithTimeout(url: string, init: RequestInit, timeoutMs: number): Promise<Response> {
  const ctrl = new AbortController()
  const t = setTimeout(() => ctrl.abort(), timeoutMs)
  try {
    return await fetch(url, { ...init, signal: ctrl.signal })
  }
  finally {
    clearTimeout(t)
  }
}

export { isWellFormedSigPayload, parseSigPayload }
