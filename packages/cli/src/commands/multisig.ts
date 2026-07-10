/**
 * `alula multisig …` — three gap-filler commands stellar-cli does not
 * cover. See ../../README.md and docs/multisig.md for the
 * runbook context.
 */

import type { Horizon } from '@stellar/stellar-sdk'
import { Keypair, Operation, Transaction, TransactionBuilder } from '@stellar/stellar-sdk'
import { Command, Option } from 'commander'
import { DEFAULT_FEE_STROOPS, DEFAULT_HOME_DOMAIN, DEFAULT_TIMEOUT_SECONDS } from '../constants'
import { useContext } from '../context'
import { blank, emit, fail, gAddressList, int, kv, matchSignaturesToSigners, ok, readStdin, step, warn } from '../utils'

export function multisigCommands(): Command {
  const multisig = new Command('multisig').description('Multisig ceremonies and audits')

  multisig.command('setup')
    .description('Configure a fresh account in one tx: add N signers, set M-of-N thresholds, burn master, set home_domain')
    .addOption(envStr('--master-secret <S>', 'Funder/master secret (S…). Omit to auto-generate an ephemeral key (testnet only — friendbot funded).', 'MASTER_SECRET'))
    .addOption(envParsed('--signers <list>', 'CSV of initial signer G-addresses', 'SIGNERS', gAddressList).makeOptionMandatory())
    .addOption(envParsed('--threshold <m>', 'M for the M-of-N threshold', 'THRESHOLD', int).makeOptionMandatory())
    .addOption(envStr('--home-domain <domain>', 'Home domain', 'HOME_DOMAIN').default(DEFAULT_HOME_DOMAIN))
    .addOption(envStr('--fee <stroops>', 'Base fee in stroops', 'FEE').default(DEFAULT_FEE_STROOPS))
    .addOption(envParsed('--timeout <seconds>', 'tx timeout in seconds', 'TIMEOUT', int).default(DEFAULT_TIMEOUT_SECONDS))
    .addOption(new Option('--dry-run', 'Print signed XDR, do not submit').env('DRY_RUN'))
    .action(runSetup)

  multisig.command('verify')
    .description('Inspect signers + thresholds; flag bricked / unsafe configs; assert expectations')
    .addOption(envStr('--account <G>', 'Multisig G-address to inspect', 'ACCOUNT').makeOptionMandatory())
    .addOption(envParsed('--expect-threshold <m>', 'Assert med_threshold equals this', 'EXPECT_THRESHOLD', int))
    .addOption(envParsed('--expect-signers <list>', 'Assert active signer set is exactly this CSV (master excluded)', 'EXPECT_SIGNERS', gAddressList))
    .action(runVerify)

  multisig.command('check-quorum')
    .description('Pre-flight a signed envelope: do the collected signatures meet the source account threshold?')
    .addOption(envStr('--xdr <xdr>', 'Signed envelope XDR (or pipe via stdin)', 'XDR'))
    .action(runCheckQuorum)

  return multisig
}

interface SetupOpts {
  masterSecret?: string
  signers: string[]
  threshold: number
  homeDomain: string
  fee: string
  timeout: number
  dryRun?: boolean
}

async function runSetup(opts: SetupOpts): Promise<void> {
  const { network, server } = useContext()

  if (opts.threshold === 0) {
    throw new Error('--threshold must be ≥ 1 — 0 would brick the account')
  }
  if (opts.threshold > opts.signers.length) {
    throw new Error(`--threshold ${opts.threshold} exceeds signer count ${opts.signers.length}`)
  }
  if (new Set(opts.signers).size !== opts.signers.length) {
    throw new Error('--signers contains duplicates')
  }

  const master = await resolveMaster(opts.masterSecret, network, server)
  kv('master/account', master.publicKey())
  kv('network', network.passphrase)
  kv('signers', `(${opts.signers.length}) ${opts.signers.join(', ')}`)
  kv('threshold', `${opts.threshold}-of-${opts.signers.length}`)
  kv('home_domain', opts.homeDomain)

  const account = await server.loadAccount(master.publicKey())
  const builder = new TransactionBuilder(account, {
    fee: opts.fee,
    networkPassphrase: network.passphrase,
  })

  for (const pk of opts.signers) {
    builder.addOperation(Operation.setOptions({
      signer: { ed25519PublicKey: pk, weight: 1 },
    }))
  }

  // Final op: set thresholds, burn master, set home domain.
  builder.addOperation(Operation.setOptions({
    masterWeight: 0,
    lowThreshold: opts.threshold,
    medThreshold: opts.threshold,
    highThreshold: opts.threshold,
    homeDomain: opts.homeDomain,
  }))

  const tx = builder.setTimeout(opts.timeout).build()
  tx.sign(master)

  if (opts.dryRun) {
    blank()
    step('DRY_RUN — printing signed XDR, NOT submitting')
    emit({
      xdr: tx.toEnvelope().toXDR('base64'),
      // surfaced so the operator can re-submit later if --dry-run was
      // run with an auto-generated master; null when --master-secret was
      // supplied (we don't echo back the operator's own key).
      ephemeral_master_secret: opts.masterSecret ? null : master.secret(),
    })
    return
  }

  blank()
  step('submitting…')
  const result = await server.submitTransaction(tx)
  ok(`submitted: ${result.hash}`)
  kv('next', `pnpm cli multisig verify --account ${master.publicKey()} --network ${network.name}`)
  emit({ hash: result.hash, ledger: result.ledger })
}

interface VerifyOpts {
  account: string
  expectThreshold?: number
  expectSigners?: string[]
}

async function runVerify(opts: VerifyOpts): Promise<void> {
  const { network, server } = useContext()
  const account = await server.loadAccount(opts.account)
  const t = account.thresholds
  const masterEntry = account.signers.find(s => s.key === opts.account)
  const activeSigners = account.signers.filter(s => s.weight > 0 && s.key !== opts.account)
  const reachableWeight = activeSigners.reduce((sum, s) => sum + s.weight, 0)

  emit({
    account: opts.account,
    network: network.passphrase,
    home_domain: account.home_domain ?? null,
    thresholds: t,
    master_weight: masterEntry?.weight ?? 0,
    signers: account.signers.map(s => ({ key: s.key, weight: s.weight, type: s.type })),
    active_signer_count: activeSigners.length,
    reachable_weight: reachableWeight,
  })

  const warnings: string[] = []
  if ((masterEntry?.weight ?? 0) > 0) {
    warnings.push(`master key has weight ${masterEntry?.weight} — burn it (set masterWeight: 0) before this account is trusted with admin authority`)
  }
  if (t.low_threshold !== t.med_threshold || t.med_threshold !== t.high_threshold) {
    warnings.push(`non-uniform thresholds (low=${t.low_threshold} med=${t.med_threshold} high=${t.high_threshold}) — Alula assumes uniform thresholds; webapp snapshots will be misleading`)
  }
  if (reachableWeight < t.med_threshold) {
    warnings.push(`BRICKED: reachable weight ${reachableWeight} < med_threshold ${t.med_threshold}. No quorum is possible.`)
  }
  if (t.med_threshold === 0) {
    warnings.push('med_threshold is 0 — any signature passes. Configure an actual threshold.')
  }
  if (opts.expectThreshold !== undefined && t.med_threshold !== opts.expectThreshold) {
    warnings.push(`assertion failed: --expect-threshold ${opts.expectThreshold} but on-chain med_threshold=${t.med_threshold}`)
  }
  if (opts.expectSigners && opts.expectSigners.length > 0) {
    const onChain = new Set(activeSigners.map(s => s.key))
    const expected = new Set(opts.expectSigners)
    const missing = [...expected].filter(k => !onChain.has(k))
    const extra = [...onChain].filter(k => !expected.has(k))
    if (missing.length) {
      warnings.push(`assertion failed: missing expected signers: ${missing.join(', ')}`)
    }
    if (extra.length) {
      warnings.push(`assertion failed: unexpected signers on-chain: ${extra.join(', ')}`)
    }
  }

  if (warnings.length > 0) {
    blank()
    for (const w of warnings) {
      warn(w)
    }
    fail(`${warnings.length} issue(s) — see above`)
  } else {
    ok(`${activeSigners.length}-signer account, ${t.med_threshold}-of-${activeSigners.length} threshold, master burned`)
  }
}

interface CheckQuorumOpts {
  xdr?: string
}

async function runCheckQuorum(opts: CheckQuorumOpts): Promise<void> {
  const { network, server } = useContext()

  const xdr = opts.xdr ?? await readStdin()
  if (!xdr) {
    fail('XDR not provided (use --xdr or pipe via stdin)')
  }

  const tx = new Transaction(xdr, network.passphrase)
  kv('source account', tx.source)
  kv('tx hash (hex)', tx.hash().toString('hex'))
  kv('signatures', tx.signatures.length)

  const account = await server.loadAccount(tx.source)
  const requiredThreshold = account.thresholds.med_threshold

  const { matchedSigners, unrecognizedHints, reachedWeight } = matchSignaturesToSigners(
    account.signers,
    tx.signatures.map(s => s.hint()),
  )

  for (const hint of unrecognizedHints) {
    warn(`signature with hint ${hint} does not match any current signer (will be rejected by core)`)
  }

  emit({
    source: tx.source,
    tx_hash: tx.hash().toString('hex'),
    required_threshold: requiredThreshold,
    reached_weight: reachedWeight,
    quorum_met: reachedWeight >= requiredThreshold,
    matched_signers: matchedSigners,
    unrecognized_signature_hints: unrecognizedHints,
  })

  if (reachedWeight < requiredThreshold) {
    fail(`weight ${reachedWeight} < required ${requiredThreshold} — collect more signatures before \`stellar tx send\``)
  }

  ok(`quorum met: ${reachedWeight}/${requiredThreshold} — safe to submit with \`stellar tx send\``)
}

// ── runtime helpers ────────────────────────────────────────────────────────

/**
 * Resolve the master keypair for `multisig setup`.
 *
 * If the operator supplied `--master-secret`, parse it and return.
 * Otherwise generate an ephemeral keypair, friendbot-fund it (testnet
 * only), and return that. The secret is logged to stderr so it appears
 * in the operator's terminal scrollback in case the setup tx fails and
 * needs to be retried — after a successful setup the secret has zero
 * authority (master is burned in the same tx) and can be discarded.
 *
 * On public, omitting `--master-secret` is an error: friendbot doesn't
 * exist on mainnet and there's no in-process way to fund the account.
 */
async function resolveMaster(
  secret: string | undefined,
  network: { name: 'testnet' | 'public' },
  server: Horizon.Server,
): Promise<Keypair> {
  if (secret) {
    return Keypair.fromSecret(secret)
  }

  if (network.name !== 'testnet') {
    throw new Error(
      '--master-secret required on public (auto-generation can only fund via friendbot, which is testnet-only). '
      + 'Generate + fund externally, then pass --master-secret.',
    )
  }

  const master = Keypair.random()
  step('generated ephemeral master keypair')
  warn(`master secret (ephemeral, will be burned in setup tx): ${master.secret()}`)
  step('funding via friendbot…')
  await server.friendbot(master.publicKey()).call()
  ok('funded')
  return master
}

// ── option helpers ─────────────────────────────────────────────────────────

/** String option with env fallback. Add `.makeOptionMandatory()` / `.default(…)` as needed. */
function envStr(flags: string, desc: string, envVar: string): Option {
  return new Option(flags, desc).env(envVar)
}

/**
 * Option with a custom parser AND env fallback. Commander's `.env()` routes
 * the env value through `.argParser()`, so envVar=`SIGNERS=a,b` works the
 * same as `--signers a,b`.
 */
function envParsed<T>(flags: string, desc: string, envVar: string, parser: (v: string) => T): Option {
  return new Option(flags, desc).env(envVar).argParser(parser)
}
