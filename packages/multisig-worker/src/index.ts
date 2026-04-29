/**
 * Alula multisig signature relay.
 *
 * Two routes:
 *   POST /sigs/:proposalHash  — append a sig payload to the KV value
 *   GET  /sigs/:proposalHash  — return all current sig payloads
 *
 * The Worker performs only structural validation. Cryptographic validation
 * happens client-side in every signer/aggregator browser. See spec §7.
 */

import { Hono } from 'hono'

interface Env {
  SIGS: KVNamespace
}

const SIG_REGEX = /^alula-sig:v1:([0-9a-f]{64}):(G[A-Z2-7]{55}):([A-Za-z0-9+/]+={0,2})$/
const HASH_REGEX = /^[0-9a-f]{64}$/
const KV_TTL_SECONDS = 30 * 24 * 3600
const MAX_SIGS_PER_PROPOSAL = 64

const app = new Hono<{ Bindings: Env }>()

app.post('/sigs/:hash', async (c) => {
  const hash = c.req.param('hash')
  if (!HASH_REGEX.test(hash)) return c.text('bad hash', 400)

  const body = (await c.req.text()).trim()
  if (!SIG_REGEX.test(body)) return c.text('bad payload', 400)

  // Bind the payload's embedded hash to the URL hash to prevent cross-proposal injection.
  const m = SIG_REGEX.exec(body)
  if (!m || m[1] !== hash) return c.text('hash mismatch', 400)

  const key = `sigs:${hash}`
  const cur = (await c.env.SIGS.get(key))?.split('\n').filter(Boolean) ?? []
  if (cur.length >= MAX_SIGS_PER_PROPOSAL) return c.text('full', 429)
  if (!cur.includes(body)) cur.push(body)
  await c.env.SIGS.put(key, cur.join('\n'), { expirationTtl: KV_TTL_SECONDS })
  return c.text('ok')
})

app.get('/sigs/:hash', async (c) => {
  const hash = c.req.param('hash')
  if (!HASH_REGEX.test(hash)) return c.text('bad hash', 400)
  const value = await c.env.SIGS.get(`sigs:${hash}`)
  return c.text(value ?? '')
})

app.get('/health', c => c.text('ok'))

export default app
