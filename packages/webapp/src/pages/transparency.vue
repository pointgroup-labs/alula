<script lang="ts" setup>
import {
  AQUA_PROVIDER_ADDRESS,
  SOROSWAP_PROVIDER_ADDRESS,
} from '@alula/client-sdk'
import aquaLogo from '~/assets/img/providers/aqua-logo.png'
import redstoneLogo from '~/assets/img/providers/redstone-logo.ico'
import reflectorLogo from '~/assets/img/providers/reflector-logo.svg'
import soroswapLogo from '~/assets/img/providers/soroswap-logo.jpg'
import { AUDITORS } from '~/config/audits'
import { ALULA_URL, DOCS_URL, GITHUB_URL } from '~/config/common'
import {
  bigintToNumber,
  formatPrice,
  truncatePercent,
} from '~/utils'

definePageMeta({
  layout: 'default',
})

const marketsStore = useMarketsStore()
const tokensStore = useTokensStore()
const toast = useToast()
const { generateExplorerLink } = useExplorerLink()

// Token registry is fetched from GitHub once and cached in the store. Kicking
// off the fetch on mount lets `getTokenIcon(symbol)` resolve real icons
// instead of empty fallbacks for late-arriving market data.
onMounted(() => {
  tokensStore.fetchTokens()
})

// Resolve `native` → `XLM` for display (the on-chain symbol for the native
// asset is the literal string `native`, but the registry keys it under both).
function tokenSymbolDisplay(symbol: string | undefined): string {
  return symbol === 'native' ? 'XLM' : symbol ?? ''
}

const loading = computed(() => marketsStore.state.loading)
const markets = computed(() =>
  Object.values(marketsStore.state.markets).filter(Boolean),
)

// Aggregate stats across every loaded pool. We don't have on-chain TVL in USD
// directly — we derive it the same way the markets table does: pool.total_supply
// × oracle_asset_price, summed. Decimal precision is handled in `bigintToNumber`
// so we never trip MAX_SAFE_INTEGER on big TVLs.
const aggregateStats = computed(() => {
  let totalSupplyUsd = 0
  let totalBorrowedUsd = 0
  let poolCount = 0
  let pricedPools = 0
  let unpricedPools = 0

  for (const market of markets.value) {
    const oracleDecimals = market.marketState?.oracle_price_decimals ?? 0
    const pools = market.marketState?.pools_data ?? []
    for (const pd of pools) {
      poolCount++
      const tokenDecimals = pd.pool.token_decimals ?? 7
      const priceRaw = pd.oracle_asset_price ?? 0n
      const priceHuman = priceRaw && oracleDecimals
        ? Number(bigintToNumber(priceRaw, oracleDecimals))
        : 0
      const supplyHuman = Number(bigintToNumber(pd.total_supply ?? 0n, tokenDecimals))
      const borrowedHuman = Number(bigintToNumber(pd.pool.total_borrowed ?? 0n, tokenDecimals))
      // Treat any pool that has supply but no price as "unpriced" — this catches
      // both genuinely stale feeds and the pre-fetch state where the oracle
      // round-trip hasn't returned yet. The headline TVL silently undercounts
      // those pools, so we surface a qualifier in the UI when it happens.
      if (supplyHuman > 0 && priceHuman === 0) {
        unpricedPools++
      } else if (supplyHuman > 0) {
        pricedPools++
      }
      totalSupplyUsd += supplyHuman * priceHuman
      totalBorrowedUsd += borrowedHuman * priceHuman
    }
  }

  return {
    marketCount: markets.value.length,
    poolCount,
    pricedPools,
    unpricedPools,
    hasStalePrices: unpricedPools > 0,
    totalSupplyUsd,
    totalBorrowedUsd,
    utilizationPct: totalSupplyUsd > 0
      ? (totalBorrowedUsd / totalSupplyUsd) * 100
      : 0,
  }
})

// Truncate a Stellar contract / account address to head + tail with an ellipsis
// in between. Stellar G/C addresses are 56 chars — fully readable they would
// blow out a card; the common convention (Stellar.expert, freighter) is 4–6
// chars on each side.
function truncateAddress(addr: string | undefined, head = 6, tail = 6): string {
  if (!addr) {
    return '-'
  }
  if (addr.length <= head + tail + 1) {
    return addr
  }
  return `${addr.slice(0, head)}…${addr.slice(-tail)}`
}

async function copyAddress(addr: string | undefined) {
  if (!addr) {
    return
  }
  try {
    await navigator.clipboard.writeText(addr)
    toast.create({ title: 'Copied', body: 'Address copied to clipboard', modelValue: 2000 })
  } catch (error) {
    toast.create({
      title: 'Copy failed',
      body: String((error as Error)?.message ?? error),
      variant: 'danger',
      modelValue: 4000,
    })
  }
}

// Flatten the four interesting addresses on a market into one uniform list so
// the template can render them with a single v-for. Keeping a stable order
// (Market → Admin → Oracle → Deployer → Insurance) lets users scan multiple
// cards without re-orienting per card.
function marketContracts(market: typeof markets.value[number]) {
  const gs = market.marketState?.global_state
  return [
    { label: 'Market', address: market.address, kind: 'contract' as const },
    { label: 'Admin', address: gs?.admin, kind: 'account' as const },
    { label: 'Oracle', address: gs?.oracle, kind: 'contract' as const },
    { label: 'Deployer', address: gs?.deployer, kind: 'account' as const },
    { label: 'Insurance', address: gs?.insurance_fund, kind: 'account' as const },
  ]
}

// Map the on-chain `MarketStatus` u32 to a human label + a short "tone" hint
// that the UI uses to switch between "ok" (active) and "warn" (any frozen
// variant) styling. Order mirrors the Rust enum in
// `contracts/market/src/storage.rs::MarketStatus`.
const MARKET_STATUS_LABELS = [
  'Active',
  'Borrow frozen',
  'Borrow frozen (admin)',
  'Deposit frozen',
  'Deposit frozen (admin)',
  'Frozen',
  'Frozen (admin)',
] as const

function marketStatusInfo(status: number | undefined) {
  const idx = typeof status === 'number' ? status : 0
  const label = MARKET_STATUS_LABELS[idx] ?? `Unknown (${idx})`
  // Three-tier severity: Active is healthy, soft-frozen variants are a
  // warning (operations curtailed but expected to recover), admin-locked
  // variants are the most severe (only Market Admin can move from them —
  // see `MarketStatus::is_admin_protected` in storage.rs).
  let tone: 'ok' | 'warn' | 'danger' = 'warn'
  if (idx === 0) {
    tone = 'ok'
  } else if (idx === 2 || idx === 4 || idx === 6) {
    tone = 'danger'
  }
  return { label, tone }
}

// Format the timelock period (seconds) as a compact human duration.
// Rounded down to the nearest hour because parameter changes that take effect
// in less than an hour are operationally indistinguishable from "immediate".
function formatTimelock(seconds: bigint | number | undefined): string | undefined {
  if (seconds == null) {
    return
  }
  const secs = Number(seconds)
  if (!Number.isFinite(secs) || secs <= 0) {
    return
  }
  const hours = Math.floor(secs / 3600)
  if (hours < 24) {
    return `${hours}h`
  }
  const days = Math.floor(hours / 24)
  const remHours = hours % 24
  return remHours > 0 ? `${days}d ${remHours}h` : `${days}d`
}

const swapProviders = [
  {
    name: 'Aquarius',
    address: AQUA_PROVIDER_ADDRESS,
    logo: aquaLogo,
    url: 'https://aqua.network',
    category: 'AMM Router',
  },
  {
    name: 'Soroswap',
    address: SOROSWAP_PROVIDER_ADDRESS,
    logo: soroswapLogo,
    url: 'https://soroswap.finance',
    category: 'AMM Router',
  },
]

// Oracle integrations are displayed statically — the on-chain Aggregated
// Oracle address is already shown per market under "Oracle", so this section
// is about *which* upstream feeds the aggregator is built on, not a directory
// of Soroban contract IDs (those rotate per asset and aren't a useful badge).
//
// Two kinds of upstream sources end up in the aggregator:
//   - Native SEP-40 feeds (e.g. Reflector) plug straight in.
//   - Non-SEP-40 feeds are wrapped by the per-vendor adapter contracts that
//     ship under `contracts/`:
//       - redstone_sep_40_adapter/  → RedStone
//       - soroswap_sep_40_adapter/  → Soroswap AMM TWAP
// The Aggregated Oracle itself (`contracts/aggregated-oracle/`) is listed
// first as the aggregation layer that combines them.
//
// `logo` is optional: when missing, the template falls back to a monogram
// avatar built from the first letter of `name`, which keeps the card layout
// identical to a logo'd one without checking in vendor artwork we don't have.
type OracleIntegration = {
  name: string
  role: string
  url: string
  logo?: string
}

const oracleIntegrations: OracleIntegration[] = [
  {
    name: 'Reflector',
    role: 'Native SEP-40 feed',
    url: 'https://reflector.network',
    logo: reflectorLogo,
  },
  {
    name: 'RedStone',
    role: 'External feed via SEP-40 adapter',
    url: 'https://redstone.finance',
    logo: redstoneLogo,
  },
  {
    name: 'Soroswap TWAP',
    role: 'DEX-based fallback via SEP-40 adapter',
    url: 'https://soroswap.finance',
    logo: soroswapLogo,
  },
  {
    name: 'Aggregated Oracle',
    role: 'Aggregation layer · median + circuit breaker',
    url: DOCS_URL,
  },
]

function monogram(name: string): string {
  return name.trim().charAt(0).toUpperCase() || '?'
}

// Multisig roster — same discriminated-union pattern as `Auditor`. Entries
// start as `pending` (no address yet) and graduate to `deployed` once the
// signer set is live on-chain. The `threshold` field is rendered as a
// fraction pill ("3/7") so the M-of-N security model is legible at a glance.
//
// The whole section currently ships pending — none of the multisigs are
// deployed on mainnet yet — so every card renders with the ghosted treatment
// (opacity 0.4, dashed border) defined on `.multisig-card--pending`.
type Multisig
  = | {
    status: 'deployed'
    name: string
    threshold: string
    description: string
    address: string
  }
  | {
    status: 'pending'
    name: string
    threshold: string
    description: string
  }

const multisigs: Multisig[] = [
  {
    status: 'pending',
    name: 'Ops Multisig',
    threshold: '3/7',
    description: 'Looks after the incentives and reward configuration & treasury operations',
  },
  {
    status: 'pending',
    name: 'Program Multisig',
    threshold: '6/10',
    description: 'Manages and authorizes program-level configurations',
  },
  {
    status: 'pending',
    name: 'Upgrade Multisig',
    threshold: '4/7',
    description: 'Handles protocol upgrade ownership with a timelock enabled',
  },
]

// Dashboards — third-party and native analytics surfaces. Same discriminated
// union as `Multisig`/`Auditor`: `live` entries carry an outbound `url`, and
// `pending` entries don't, so the type system enforces the contract once a
// dashboard goes live. Currently every entry is pending — no public dashboards
// have been launched yet — so all three render with the ghosted treatment.
type Dashboard
  = | { status: 'live', name: string, description: string, url: string }
    | { status: 'pending', name: string, description: string }

const dashboards: Dashboard[] = [
  {
    status: 'pending',
    name: 'DefiLlama',
    description: 'TVL, fees, and cross-protocol comparisons via the public DeFi index',
  },
  {
    status: 'pending',
    name: 'Dune Analytics',
    description: 'Custom queries on supply, borrow, liquidation, and oracle activity',
  },
  {
    status: 'pending',
    name: 'Alula Stats',
    description: 'Native protocol dashboard with per-pool charts and risk metrics',
  },
]
</script>

<template>
  <main class="transparency-page container">
    <header class="transparency-page__hero">
      <span class="transparency-page__eyebrow">Transparency</span>
      <h1 class="transparency-page__title">
        Verifiable by design
      </h1>
      <p class="transparency-page__lead">
        Every contract, oracle, risk parameter, and audit that governs your
        funds is published here, so anyone can verify exactly what the
        protocol does.
      </p>

      <div class="transparency-page__hero-stats">
        <div class="transparency-page__hero-stat">
          <span class="transparency-page__hero-stat-label">Total supplied</span>
          <span class="transparency-page__hero-stat-value">
            ${{ formatPrice(aggregateStats.totalSupplyUsd, 0, 2) }}
          </span>
          <span
            v-if="aggregateStats.hasStalePrices"
            class="transparency-page__hero-stat-note"
            :title="`${aggregateStats.unpricedPools} of ${aggregateStats.pricedPools + aggregateStats.unpricedPools} pools have no price right now. Total excludes them.`"
          >
            {{ aggregateStats.pricedPools }}/{{ aggregateStats.pricedPools + aggregateStats.unpricedPools }} pools priced
          </span>
        </div>
        <div class="transparency-page__hero-stat">
          <span class="transparency-page__hero-stat-label">Total borrowed</span>
          <span class="transparency-page__hero-stat-value">
            ${{ formatPrice(aggregateStats.totalBorrowedUsd, 0, 2) }}
          </span>
        </div>
        <div class="transparency-page__hero-stat">
          <span class="transparency-page__hero-stat-label">Utilization</span>
          <span class="transparency-page__hero-stat-value">
            {{ truncatePercent(aggregateStats.utilizationPct, 2) }}%
          </span>
        </div>
        <div class="transparency-page__hero-stat">
          <span class="transparency-page__hero-stat-label">Markets / Pools</span>
          <span class="transparency-page__hero-stat-value">
            {{ aggregateStats.marketCount }} / {{ aggregateStats.poolCount }}
          </span>
        </div>
      </div>
    </header>

    <section
      id="code"
      class="transparency-section"
    >
      <header class="transparency-section__header">
        <h2 class="transparency-section__title">
          Source code
        </h2>
        <p class="transparency-section__subtitle">
          Every Soroban contract, the SDKs, and this webapp live in a single
          public monorepo. Read the code, run the tests, file an issue.
        </p>
      </header>
      <a
        class="code-card"
        :href="GITHUB_URL"
        target="_blank"
        rel="noopener noreferrer"
      >
        <span
          class="code-card__icon"
          aria-hidden="true"
        >
          <i-app-github />
        </span>
        <span class="code-card__body">
          <span class="code-card__eyebrow">GitHub</span>
          <span class="code-card__title">Alula Protocol codebase</span>
          <span class="code-card__desc">
            Soroban contracts, SDKs, and webapp
          </span>
        </span>
        <span class="code-card__cta">
          View on GitHub
          <i-app-export-icon />
        </span>
      </a>
    </section>

    <section
      id="audits"
      class="transparency-section"
    >
      <header class="transparency-section__header">
        <h2 class="transparency-section__title">
          Audits
        </h2>
        <p class="transparency-section__subtitle">
          Independent security firms audit every release before it ships.
          Their reports are linked below, unredacted.
        </p>
      </header>

      <div class="transparency-section__grid transparency-section__grid--audits">
        <article
          v-for="auditor in AUDITORS"
          :key="auditor.name"
          class="audit-card"
          :class="{ 'audit-card--pending': auditor.status === 'pending' }"
        >
          <img
            :src="auditor.logo"
            :alt="`${auditor.name} logo`"
            class="audit-card__logo"
          >
          <div class="audit-card__body">
            <span class="audit-card__name">{{ auditor.name }}</span>
            <span
              v-if="auditor.status === 'completed'"
              class="audit-card__date"
            >Audited {{ auditor.auditedAt }}</span>
            <span
              v-else
              class="audit-card__date"
            >Audit in progress</span>
          </div>
          <a
            v-if="auditor.status === 'completed'"
            class="audit-card__link"
            :href="auditor.link"
            target="_blank"
            rel="noopener noreferrer"
          >
            Read report
            <i-app-export-icon />
          </a>
        </article>
      </div>
    </section>

    <section
      id="markets"
      class="transparency-section"
    >
      <header class="transparency-section__header">
        <h2 class="transparency-section__title">
          Live markets
        </h2>
        <p class="transparency-section__subtitle">
          Loaded directly from the on-chain state of every Market contract this
          app talks to. Refresh to re-pull.
        </p>
      </header>

      <div
        v-if="loading && markets.length === 0"
        class="transparency-section__empty"
      >
        Loading on-chain market data…
      </div>

      <div
        v-else-if="markets.length === 0"
        class="transparency-section__empty"
      >
        No markets discovered. Connect to a configured network to see them.
      </div>

      <div
        v-else
        class="transparency-section__stack"
      >
        <article
          v-for="market in markets"
          :key="market.address"
          class="market-card"
        >
          <div class="market-card__title-row">
            <h3 class="market-card__name">
              {{ market.marketName || 'Unnamed market' }}
            </h3>
            <span
              class="market-card__status"
              :class="`market-card__status--${marketStatusInfo(market.marketState?.global_state?.status).tone}`"
              :title="`Market status: ${marketStatusInfo(market.marketState?.global_state?.status).label}`"
            >
              <span
                class="market-card__status-dot"
                aria-hidden="true"
              />
              {{ marketStatusInfo(market.marketState?.global_state?.status).label }}
            </span>
            <span
              class="market-card__sep"
              aria-hidden="true"
            >·</span>
            <span class="market-card__pool-count">
              {{ market.marketState?.pools_data?.length ?? 0 }}
              pool{{ (market.marketState?.pools_data?.length ?? 0) === 1 ? '' : 's' }}
            </span>
            <template v-if="formatTimelock(market.marketState?.global_state?.update_in_queue_period)">
              <span
                class="market-card__sep"
                aria-hidden="true"
              >·</span>
              <span
                class="market-card__meta"
                title="Parameter changes are queued for this duration before they take effect"
              >
                {{ formatTimelock(market.marketState?.global_state?.update_in_queue_period) }} timelock
              </span>
            </template>
            <template v-if="market.marketState?.global_state && market.marketState.global_state.is_owned === false">
              <span
                class="market-card__sep"
                aria-hidden="true"
              >·</span>
              <span
                class="market-card__meta market-card__meta--strong"
                title="Admin powers have been renounced. No privileged calls are possible."
              >
                Renounced
              </span>
            </template>
            <ul
              v-if="(market.marketState?.pools_data?.length ?? 0) > 0"
              class="market-card__assets"
            >
              <li
                v-for="pd in market.marketState!.pools_data"
                :key="pd.pool.pool_address"
              >
                <a
                  class="market-card__asset-chip"
                  :href="generateExplorerLink(pd.pool.token_address, 'contract')"
                  target="_blank"
                  rel="noopener noreferrer"
                  :title="pd.pool.token_address"
                >
                  <img
                    v-if="tokensStore.getTokenIcon(pd.pool.token_symbol)"
                    :src="tokensStore.getTokenIcon(pd.pool.token_symbol)"
                    :alt="`${tokenSymbolDisplay(pd.pool.token_symbol)} icon`"
                    class="market-card__asset-icon"
                    loading="lazy"
                    width="14"
                    height="14"
                  >
                  {{ tokenSymbolDisplay(pd.pool.token_symbol) }}
                </a>
              </li>
            </ul>
          </div>

          <ul class="market-card__contracts">
            <li
              v-for="entry in marketContracts(market)"
              :key="entry.label"
              class="contract-chip"
            >
              <span class="contract-chip__label">{{ entry.label }}</span>
              <a
                class="contract-chip__addr"
                :href="generateExplorerLink(entry.address ?? '', entry.kind)"
                target="_blank"
                rel="noopener noreferrer"
                :title="entry.address"
              >{{ truncateAddress(entry.address, 4, 4) }}</a>
              <button
                type="button"
                class="contract-chip__copy"
                :aria-label="`Copy ${entry.label} address`"
                @click="copyAddress(entry.address)"
              >
                <i-app-copy />
              </button>
            </li>
          </ul>
        </article>
      </div>
    </section>

    <section
      id="swap-providers"
      class="transparency-section"
    >
      <header class="transparency-section__header">
        <h2 class="transparency-section__title">
          Swap providers
        </h2>
        <p class="transparency-section__subtitle">
          The Market contract routes swaps through these on-chain venues.
          Quotes are queried in parallel and ranked best-first.
        </p>
      </header>
      <div class="transparency-section__grid transparency-section__grid--providers">
        <article
          v-for="provider in swapProviders"
          :key="provider.name"
          class="provider-card"
        >
          <a
            class="provider-card__main"
            :href="provider.url"
            target="_blank"
            rel="noopener noreferrer"
          >
            <img
              :src="provider.logo"
              :alt="`${provider.name} logo`"
              class="provider-card__logo"
              loading="lazy"
              width="32"
              height="32"
            >
            <span class="provider-card__heading">
              <span class="provider-card__name">{{ provider.name }}</span>
              <span class="provider-card__meta">
                {{ provider.category }} · Live
              </span>
            </span>
            <i-app-export-icon class="provider-card__ext" />
          </a>
          <span class="address-pill provider-card__addr">
            <a
              class="address-pill__addr"
              :href="generateExplorerLink(provider.address, 'contract')"
              target="_blank"
              rel="noopener noreferrer"
              :title="provider.address"
            >{{ truncateAddress(provider.address) }}</a>
            <button
              type="button"
              class="address-pill__copy"
              :aria-label="`Copy ${provider.name} provider address`"
              @click="copyAddress(provider.address)"
            >
              <i-app-copy />
            </button>
          </span>
        </article>
      </div>
    </section>

    <section
      id="oracles"
      class="transparency-section"
    >
      <header class="transparency-section__header">
        <h2 class="transparency-section__title">
          Oracle integrations
        </h2>
        <p class="transparency-section__subtitle">
          Oracles deliver the real-time prices that drive risk checks and
          liquidations. Multiple independent SEP-40 sources are combined by
          the protocol's own Aggregated Oracle so the system stays solvent
          even when a single feed degrades.
        </p>
      </header>
      <div class="transparency-section__grid transparency-section__grid--providers">
        <article
          v-for="oracle in oracleIntegrations"
          :key="oracle.name"
          class="provider-card"
        >
          <a
            class="provider-card__main"
            :href="oracle.url"
            target="_blank"
            rel="noopener noreferrer"
          >
            <img
              v-if="oracle.logo"
              :src="oracle.logo"
              :alt="`${oracle.name} logo`"
              class="provider-card__logo"
              loading="lazy"
              width="32"
              height="32"
            >
            <span
              v-else
              class="provider-card__logo provider-card__logo--mono"
              aria-hidden="true"
            >
              {{ monogram(oracle.name) }}
            </span>
            <span class="provider-card__heading">
              <span class="provider-card__name">{{ oracle.name }}</span>
              <span class="provider-card__meta">
                {{ oracle.role }} · Integration
              </span>
            </span>
            <i-app-export-icon class="provider-card__ext" />
          </a>
        </article>
      </div>
    </section>

    <section
      id="multisigs"
      class="transparency-section"
    >
      <header class="transparency-section__header">
        <h2 class="transparency-section__title">
          Multisignature wallets
        </h2>
        <p class="transparency-section__subtitle">
          Multisig wallets safeguard protocol governance, upgrades, and fund
          movements. Each action requires coordinated approval from trusted
          signers, giving Alula decentralization, transparency, and
          defense-in-depth.
        </p>
      </header>
      <div class="transparency-section__grid transparency-section__grid--multisigs">
        <article
          v-for="ms in multisigs"
          :key="ms.name"
          class="multisig-card"
          :class="{ 'multisig-card--pending': ms.status === 'pending' }"
        >
          <div class="multisig-card__head">
            <span class="multisig-card__name">{{ ms.name }}</span>
            <span
              class="multisig-card__threshold"
              :title="`${ms.threshold} signers required`"
            >{{ ms.threshold }}</span>
          </div>
          <p class="multisig-card__desc">
            {{ ms.description }}
          </p>
          <span
            v-if="ms.status === 'pending'"
            class="multisig-card__status"
          >Pending deployment</span>
        </article>
      </div>
    </section>

    <section
      id="dashboards"
      class="transparency-section"
    >
      <header class="transparency-section__header">
        <h2 class="transparency-section__title">
          Dashboards
        </h2>
        <p class="transparency-section__subtitle">
          Public analytics for protocol activity, risk, and reserves. External
          and native surfaces will publish here as they go live.
        </p>
      </header>
      <div class="transparency-section__grid transparency-section__grid--multisigs">
        <component
          :is="dash.status === 'live' ? 'a' : 'article'"
          v-for="dash in dashboards"
          :key="dash.name"
          class="dashboard-card"
          :class="{ 'dashboard-card--pending': dash.status === 'pending' }"
          :href="dash.status === 'live' ? dash.url : undefined"
          :target="dash.status === 'live' ? '_blank' : undefined"
          :rel="dash.status === 'live' ? 'noopener noreferrer' : undefined"
        >
          <div class="dashboard-card__head">
            <span class="dashboard-card__name">{{ dash.name }}</span>
            <i-app-export-icon
              v-if="dash.status === 'live'"
              class="dashboard-card__ext"
            />
          </div>
          <p class="dashboard-card__desc">
            {{ dash.description }}
          </p>
          <span
            v-if="dash.status === 'pending'"
            class="dashboard-card__status"
          >Pending launch</span>
        </component>
      </div>
    </section>

    <section
      id="resources"
      class="transparency-section"
    >
      <header class="transparency-section__header">
        <h2 class="transparency-section__title">
          Resources
        </h2>
        <p class="transparency-section__subtitle">
          Documentation and project home. Bug bounty and disclosure channels
          will be added here as they go live.
        </p>
      </header>
      <div class="transparency-section__grid transparency-section__grid--links">
        <a
          class="link-card"
          :href="DOCS_URL"
          target="_blank"
          rel="noopener noreferrer"
        >
          <span class="link-card__title">Documentation</span>
          <span class="link-card__desc">Concepts, guides, and protocol reference.</span>
          <i-app-export-icon class="link-card__icon" />
        </a>
        <a
          class="link-card"
          :href="ALULA_URL"
          target="_blank"
          rel="noopener noreferrer"
        >
          <span class="link-card__title">alula.finance</span>
          <span class="link-card__desc">Project home and announcements.</span>
          <i-app-export-icon class="link-card__icon" />
        </a>
      </div>
    </section>
  </main>
</template>

<style lang="scss">
.transparency-page {
  padding: 32px 16px 64px;
  display: flex;
  flex-direction: column;
  gap: 32px;

  &__hero {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding-bottom: 24px;
    border-bottom: 1px solid $border-primary;
  }

  &__eyebrow {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.08em;
    color: $cyan;
    text-transform: uppercase;
  }

  &__title {
    font-size: 32px;
    font-weight: 700;
    color: $navi-25;
    margin: 0;
    line-height: 1.15;

    @media (max-width: 640px) {
      font-size: 26px;
    }
  }

  &__lead {
    font-size: 14px;
    color: $text-secondary;
    line-height: 1.55;
    max-width: 640px;
    margin: 0;
  }

  &__hero-stats {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 12px;
    margin-top: 12px;

    @media (max-width: 720px) {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  &__hero-stat {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px 14px;
    background-color: color-mix(in oklab, $navi-700 60%, transparent);
    border: 1px solid $border-primary;
    border-radius: $radius-lg;
  }

  &__hero-stat-label {
    font-size: 11px;
    color: $text-tertiary;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  &__hero-stat-value {
    font-size: 18px;
    font-weight: 600;
    color: $text-primary;
    font-family: $font-JetBrainsMono;
  }

  &__hero-stat-note {
    // Surfaced only when one or more pools lack a live price. Sits below the
    // headline value as a small warning-toned chip so the missing data is
    // disclosed at the same visual level as the number itself.
    margin-top: 4px;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: $warning;
    cursor: help;
  }
}

.transparency-section {
  display: flex;
  flex-direction: column;
  gap: 16px;

  &__header {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  &__title {
    font-size: 18px;
    font-weight: 700;
    color: $text-primary;
    margin: 0;
  }

  &__subtitle {
    font-size: 12px;
    color: $text-tertiary;
    margin: 0;
    line-height: 1.5;
    max-width: 640px;
  }

  &__empty {
    padding: 24px;
    text-align: center;
    color: $text-tertiary;
    font-size: 13px;
    border: 1px dashed $border-primary;
    border-radius: $radius-lg;
  }

  &__stack {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  &__grid {
    display: grid;
    gap: 12px;

    &--audits {
      grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    }
    &--providers {
      grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    }
    &--links {
      grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    }
    &--multisigs {
      // Wider min so the description has room to breathe; the threshold pill
      // sits on the same line as the name and shouldn't compete for width.
      grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    }
  }
}

// Reusable address pill — used in market cards, pool tables, swap providers.
// `min-width: 0` on the anchor lets the truncated text shrink inside a flex
// row without the copy button getting pushed off-screen on narrow cards.
.address-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 2px 6px;
  background-color: color-mix(in oklab, $navi-700 70%, transparent);
  border: 1px solid $border-primary;
  border-radius: $radius-md;
  font-family: $font-JetBrainsMono;
  font-size: 12px;
  max-width: 100%;

  &__addr {
    color: $text-primary;
    text-decoration: none;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;

    &:hover {
      color: $cyan;
    }
  }

  &__copy {
    background: none;
    border: none;
    cursor: pointer;
    color: $text-tertiary;
    display: inline-flex;
    align-items: center;
    padding: 0;

    svg {
      width: 12px;
      height: 12px;
    }

    &:hover {
      color: $text-primary;
    }
  }
}

.kv {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  font-size: 12px;

  &__k {
    color: $text-tertiary;
  }
  &__v {
    color: $text-primary;
    font-family: $font-JetBrainsMono;
  }

  &--small {
    font-size: 11px;
  }
}

.code-card {
  // Single full-width card. Three columns: GitHub mark, body block (eyebrow
  // + title + desc), CTA pill. Layout collapses to stacked on narrow widths
  // so the desc and CTA don't crash into each other on mobile.
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 18px 20px;
  background-color: $bg-card;
  border: 1px solid $border-secondary;
  border-radius: $radius-lg;
  color: inherit;
  text-decoration: none;
  transition:
    border-color $transition-base ease,
    background-color 0.12s ease;

  &:hover {
    border-color: $border-primary;
    background-color: $navi-600;
  }

  &__icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    border-radius: $radius-md;
    background-color: $navi-700;
    color: $text-primary;
    flex-shrink: 0;

    > svg {
      width: 22px;
      height: 22px;
    }
  }

  &__body {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }

  &__eyebrow {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: $text-tertiary;
  }

  &__title {
    font-size: 15px;
    font-weight: 600;
    color: $text-primary;
    line-height: 1.2;
  }

  &__desc {
    font-size: 12px;
    color: $text-tertiary;
    line-height: 1.4;
  }

  // CTA pill on the right. Uses the brand cyan to match the audit card's
  // "Read report" link, so external-out actions read consistently across
  // the page. The arrow icon nudges on hover via the parent rule below.
  &__cta {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 8px 14px;
    border: 1px solid color-mix(in oklab, $cyan 35%, $border-secondary);
    border-radius: 999px;
    font-size: 12px;
    font-weight: 600;
    color: $cyan;
    flex-shrink: 0;
    transition: border-color 0.12s ease;

    > svg {
      width: 12px;
      height: 12px;
      transition: transform $transition-base ease;
    }
  }

  &:hover &__cta {
    border-color: $cyan;

    > svg {
      transform: translate(2px, -2px);
    }
  }

  // Mobile: stack icon + body, pull the CTA below them. Same content, looser
  // rhythm so nothing overlaps inside a narrow column.
  @media (max-width: 640px) {
    flex-wrap: wrap;

    &__cta {
      margin-left: 56px; // align under body, past the 44px icon + 12px gap
    }
  }
}

.dashboard-card {
  // Mirrors `.multisig-card` minus the threshold pill — same vertical layout,
  // same pending-state vocabulary. Lives separately so dashboards can grow
  // independently (per-source filters, screenshots, embeds) without dragging
  // the multisig card schema along.
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 14px 16px;
  background-color: $bg-card;
  border: 1px solid $border-secondary;
  border-radius: $radius-lg;
  color: inherit;
  text-decoration: none;
  transition:
    border-color $transition-base ease,
    opacity $transition-base ease,
    border-style $transition-base ease,
    background-color 0.12s ease;

  // Hover affordance only fires on the live (clickable) card variant. Pending
  // entries get their own hover treatment via `--pending`, which lifts the
  // ghosting; the background hover here would fight that.
  &:not(.dashboard-card--pending):hover {
    border-color: $border-primary;
    background-color: $navi-600;
  }

  &__head {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  &__name {
    font-size: 14px;
    font-weight: 600;
    color: $text-primary;
    line-height: 1.2;
    flex: 1;
    min-width: 0;
  }

  &__ext {
    width: 12px;
    height: 12px;
    color: $text-tertiary;
    flex-shrink: 0;
    transition: transform $transition-base ease;
  }

  &:hover &__ext {
    transform: translate(2px, -2px);
    color: $text-primary;
  }

  &__desc {
    font-size: 12px;
    color: $text-secondary;
    line-height: 1.45;
    margin: 0;
    flex: 1;
  }

  &__status {
    align-self: flex-start;
    padding: 2px 8px;
    border: 1px solid $border-secondary;
    border-radius: 999px;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: $text-tertiary;
  }

  &--pending {
    opacity: 0.4;
    border-style: dashed;

    &:hover {
      opacity: 1;
      border-style: solid;
      border-color: $border-primary;
    }
  }
}

.multisig-card {
  // Vertical card with three rows: head (name + threshold), description,
  // and an optional status footer for pending entries. Description gets
  // `flex: 1` so cards in the same grid row align their footers regardless
  // of how long each individual description is.
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 14px 16px;
  background-color: $bg-card;
  border: 1px solid $border-secondary;
  border-radius: $radius-lg;
  transition:
    border-color $transition-base ease,
    opacity $transition-base ease,
    border-style $transition-base ease;

  &:hover {
    border-color: $border-primary;
  }

  &__head {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  &__name {
    font-size: 14px;
    font-weight: 600;
    color: $text-primary;
    line-height: 1.2;
    flex: 1;
    min-width: 0;
  }

  // M-of-N pill rendered in monospace so the digits sit at consistent
  // widths across cards (3/7 vs 6/10 vs 4/7 — different glyph counts).
  // Cyan accent makes the security threshold the visual anchor of each
  // card without resorting to a heavy fill.
  &__threshold {
    display: inline-flex;
    align-items: center;
    padding: 2px 8px;
    border: 1px solid color-mix(in oklab, $cyan 35%, $border-secondary);
    border-radius: 999px;
    font-family: $font-JetBrainsMono;
    font-size: 11px;
    font-weight: 600;
    color: $cyan;
    flex-shrink: 0;
  }

  &__desc {
    font-size: 12px;
    color: $text-secondary;
    line-height: 1.45;
    margin: 0;
    flex: 1;
  }

  // Pending footer pill — same uppercase eyebrow vocabulary as the market
  // status pill so the page has one visual language for "future state".
  &__status {
    align-self: flex-start;
    padding: 2px 8px;
    border: 1px solid $border-secondary;
    border-radius: 999px;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: $text-tertiary;
  }

  // Pending state: ghost the whole card to 0.4 + dashed border, mirroring
  // the audit-card pending treatment so "future state" reads consistently
  // across the page. Hover lifts to full opacity + solid border so the
  // detail copy stays discoverable on demand.
  &--pending {
    opacity: 0.4;
    border-style: dashed;

    &:hover {
      opacity: 1;
      border-style: solid;
      border-color: $border-primary;
    }
  }
}

.audit-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px;
  background-color: $bg-card;
  border: 1px solid $border-secondary;
  border-radius: $radius-lg;

  &__logo {
    width: 36px;
    height: 36px;
    border-radius: $radius-md;
    object-fit: contain;
    background-color: $navi-700;
    padding: 4px;
    flex-shrink: 0;
  }

  &__body {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }

  &__name {
    font-size: 14px;
    font-weight: 600;
    color: $text-primary;
  }

  &__date {
    font-size: 11px;
    color: $text-tertiary;
  }

  &__link {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    font-weight: 500;
    color: $cyan;
    text-decoration: none;
    flex-shrink: 0;

    svg {
      width: 12px;
      height: 12px;
    }

    &:hover {
      text-decoration: underline;
    }
  }

  // Pending state: ghost the whole card to 0.4 so the user reads "future"
  // at a glance, with a dashed border reinforcing the not-yet-active feel.
  // Hover lifts the card back to full opacity + solid border so the auditor
  // name and the "Audit in progress" copy stay discoverable on demand.
  // Transition smooths both axes; `border-style` is listed for documentation
  // even though browsers swap it discretely (not animated).
  &--pending {
    opacity: 0.4;
    border-style: dashed;
    transition:
      opacity $transition-base ease,
      border-color $transition-base ease,
      border-style $transition-base ease;

    &:hover {
      opacity: 1;
      border-style: solid;
      border-color: $border-primary;
    }
  }
}

.market-card {
  background-color: $bg-card;
  border: 1px solid $border-secondary;
  border-radius: $radius-lg;
  // Two stacked rows: identity (line 1) and contracts (line 2). Both rows
  // are flex-wrap so on narrow viewports content wraps gracefully without a
  // media query — each chip is self-contained and survives any column width.
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px 14px;
  transition: border-color $transition-base ease;

  &:hover {
    border-color: $border-primary;
  }

  &__title-row {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
    min-width: 0;
  }

  &__name {
    font-size: 14px;
    font-weight: 600;
    color: $text-primary;
    line-height: 1.2;
    margin: 0;
    min-width: 0;
    // Capitalize each word so registry names like "alula mainnet" render as
    // "Alula Mainnet" without the data layer having to normalize casing.
    text-transform: capitalize;
  }

  // Visual middot between name and pool count — tertiary color so it reads
  // as punctuation, not as data. Hidden from a11y tree.
  &__sep {
    color: $text-tertiary;
    font-size: 12px;
    line-height: 1;
  }

  &__pool-count {
    font-size: 11px;
    color: $text-tertiary;
  }

  // Generic inline meta text used for timelock + renounced indicators.
  // `--strong` variant bumps weight + color when the meta deserves more
  // attention (e.g. "Renounced" — a notable governance state).
  &__meta {
    font-size: 11px;
    color: $text-tertiary;

    &--strong {
      color: $text-primary;
      font-weight: 600;
    }
  }

  // Status pill — outline-only, no fill. Each tone tints the text and border
  // with a semantic palette token; the inner dot uses `currentColor` so it
  // inherits the tone automatically. Borders are mixed against
  // `$border-secondary` so the chip stays calm on the dark card background
  // (a full-strength accent border reads as alarming for routine states).
  &__status {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 2px 8px;
    border: 1px solid $border-secondary;
    border-radius: 999px;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: $text-tertiary;

    &--ok {
      color: $success;
      border-color: color-mix(in oklab, $success 35%, $border-secondary);
    }

    &--warn {
      color: $warning;
      border-color: color-mix(in oklab, $warning 40%, $border-secondary);
    }

    &--danger {
      color: $danger;
      border-color: color-mix(in oklab, $danger 45%, $border-secondary);
    }
  }

  &__status-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background-color: currentColor;
    box-shadow: 0 0 0 1px currentColor;
    opacity: 0.85;
  }

  &__assets {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    // Push asset chips to the right side of the title row when space allows;
    // they wrap underneath on narrow widths.
    margin-left: auto;
  }

  &__asset-chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 1px 8px 1px 4px;
    border: 1px solid $border-secondary;
    border-radius: 999px;
    font-size: 10px;
    font-weight: 600;
    color: $text-primary;
    text-decoration: none;
    font-family: $font-JetBrainsMono;
    transition:
      border-color 0.12s ease,
      background-color 0.12s ease;

    &:hover {
      border-color: $border-primary;
      background-color: $navi-600;
    }
  }

  // 14px round token icon. `background-color` gives a fallback fill if the
  // registry image 404s; `object-fit: cover` keeps non-square sources tidy.
  &__asset-icon {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    object-fit: cover;
    background-color: $navi-700;
    flex-shrink: 0;
  }

  // Contracts row: 4 inline pills, each containing label + address + copy.
  // Uses flex-wrap so up to 4 fit on one line on wide cards, 2 per line on
  // medium, all stacked on tight phones — no media queries needed.
  &__contracts {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
}

// Unified contract chip: tiny uppercase label fused with the truncated
// address and a copy button inside one rounded outline. Replaces the old
// "label + address-pill" pair so each contract is one self-contained unit
// that wraps as a single token in flex-wrap rows.
.contract-chip {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 3px 4px 3px 9px;
  border: 1px solid $border-secondary;
  border-radius: 999px;
  font-size: 11px;
  background-color: $bg-card;
  transition: border-color 0.12s ease;

  &:hover {
    border-color: $border-primary;
  }

  &__label {
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: $text-tertiary;
  }

  &__addr {
    font-family: $font-JetBrainsMono;
    color: $text-primary;
    text-decoration: none;
    font-size: 11px;

    &:hover {
      color: $cyan;
    }
  }

  &__copy {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    padding: 0;
    border: none;
    border-radius: 50%;
    background: transparent;
    cursor: pointer;
    color: $text-tertiary;
    transition:
      color 0.12s ease,
      background-color 0.12s ease;

    &:hover {
      color: $text-primary;
      background-color: $navi-600;
    }

    > svg {
      width: 11px;
      height: 11px;
    }
  }
}

.provider-card {
  // Compact two-zone card: clickable identity row on top, address pill
  // tucked underneath. No description, no CTA bar — the whole header IS
  // the link to the provider's site, and the export icon signals it.
  display: flex;
  flex-direction: column;
  background-color: $bg-card;
  border: 1px solid $border-secondary;
  border-radius: $radius-lg;
  overflow: hidden;
  transition: border-color $transition-base ease;

  &:hover {
    border-color: $border-primary;
  }

  &__main {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    color: $text-primary;
    text-decoration: none;
    transition: background-color 0.12s ease;
    // Fill the card vertically so hover-background paints edge-to-edge even
    // when the role/meta text is only one line. Without this, cards in the
    // same grid row stretch to match the tallest sibling but `__main` keeps
    // its content height — leaving an unhighlighted strip below it on hover.
    flex: 1;

    // Divider between the identity row and the address pill below it. Scoped
    // with `:not(:last-child)` so it auto-disables on cards that don't render
    // an address pill (oracle integrations) — without this guard the rule
    // collides with the article's outer border and paints a double line.
    &:not(:last-child) {
      border-bottom: 1px solid $border-secondary;
    }

    &:hover {
      background-color: $navi-600;
    }
  }

  &__logo {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    object-fit: cover;
    background-color: $navi-700;
    border: 1px solid $border-secondary;
    flex-shrink: 0;
  }

  // Monogram fallback used when an oracle integration ships without an
  // `<img>` logo asset. Matches the size/shape of the real logo so the row
  // alignment doesn't shift between cards that have and don't have artwork.
  &__logo--mono {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 13px;
    font-weight: 700;
    color: $text-secondary;
    background-color: $navi-600;
    text-transform: uppercase;
  }

  &__heading {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }

  &__name {
    font-size: 13px;
    font-weight: 600;
    color: $text-primary;
    line-height: 1.1;
  }

  &__meta {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: $text-tertiary;
  }

  &__ext {
    width: 12px;
    height: 12px;
    color: $text-tertiary;
    flex-shrink: 0;
    transition:
      transform $transition-base ease,
      color 0.12s ease;
  }

  &__main:hover &__ext {
    transform: translate(2px, -2px);
    color: $text-primary;
  }

  &__addr {
    margin: 8px 12px;
    align-self: flex-start;
  }
}

.link-card {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 14px;
  background-color: $bg-card;
  border: 1px solid $border-secondary;
  border-radius: $radius-lg;
  text-decoration: none;
  position: relative;
  transition: border-color $transition-base ease;

  &:hover {
    border-color: $navi-300;
  }

  &__title {
    font-size: 14px;
    font-weight: 600;
    color: $text-primary;
  }

  &__desc {
    font-size: 12px;
    color: $text-tertiary;
    line-height: 1.4;
  }

  &__icon {
    position: absolute;
    top: 12px;
    right: 12px;
    width: 14px;
    height: 14px;
    color: $text-tertiary;
  }
}
</style>
