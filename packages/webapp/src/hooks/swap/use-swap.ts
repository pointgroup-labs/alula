import type { AlulaClient, SwapRoute } from '@alula/client-sdk'
import { TRANSACTION_TIMEOUT } from '~/config'
import { destructurePoolAsset } from '~/utils'
import { buildObligationKey } from '~/utils/obligation'

export type SwapTokenOption = {
  /** Display symbol — `'native'` is normalized to `'XLM'`. */
  symbol: string
  /** Real token name from the registry, e.g. `'Stellar'`, `'USD Coin'`. */
  name: string
  icon: string
  tokenAddress: string
  tokenDecimals: number
  isNative: boolean
  tokenSymbol: string
  /** Classic-asset issuer for `getAssetBalance` lookup; undefined for XLM. */
  assetIssuer?: string
  /** Oracle USD price (≥ 0). Zero means unpriced — display gracefully. */
  price: number
}

const DEFAULT_SLIPPAGE_PERCENT = 0.5
// Stellar accounts must keep a base reserve plus per-subentry reserves liquid;
// matching the value used in supply-dialog.ts so "spendable" XLM is consistent
// across the app. A flat 2-XLM cushion is conservative for typical accounts
// (1 XLM base + headroom for trustlines + tx fees) and avoids a Horizon round
// trip to compute the precise reserve.
export const XLM_NATIVE_RESERVE = 2
// Re-quote cadence while the page is open so prices don't get stale on the
// user. 15s is a compromise between provider load and freshness; we also pause
// while the tab is hidden via `useDocumentVisibility`.
const QUOTE_REFRESH_INTERVAL_MS = 15_000
// Input-change debounce. Mirrors typical AMM UI snappiness — long enough to
// coalesce keystrokes, short enough that the quote feels live.
const QUOTE_DEBOUNCE_MS = 250

const DEFAULT_FROM_TOKEN_SYMBOL = 'XLM'
const DEFAULT_TO_TOKEN_SYMBOL = 'USDC'

// Dev-only diagnostics. Always-on console output leaks router state into prod
// users' consoles and is noisy during routine use; gate every log/warn behind
// this so the prod bundle stays quiet.
const DEV = import.meta.dev ?? false
function devLog(...args: unknown[]): void {
  if (DEV) {
    console.log(...args)
  }
}
function devWarn(...args: unknown[]): void {
  if (DEV) {
    console.warn(...args)
  }
}

/**
 * Standalone swap composable. The Swap page is a global converter independent of
 * the lending/multiply flows: it submits a single `SwapExactTokens` request
 * through any Market contract (the contract just forwards to the chosen swap
 * provider — see `contracts/market/src/processors.rs::process_swap_exact`).
 *
 * Token universe is the union of pool tokens across every loaded market, so the
 * Swap page only ever offers assets the protocol already supports.
 */
export function useSwap() {
  const marketsStore = useMarketsStore()
  const multiplyStore = useMultiplyStore()
  const wallet = useWalletComposable()
  const { getFullTokenData } = useTokensStore()
  const connectionStore = useConnectionStore()
  const { publicKey, nativeBalance, getAssetBalance } = useWalletComposable()
  const toast = useToast()

  const kit = computed(() => connectionStore.kit)

  // Token universe = every distinct (token_address, token_decimals) that appears
  // in any pool of any market. We dedupe on token_address — different markets may
  // surface the same token under different pool addresses, but the Swap page
  // doesn't care about pools, only about the underlying asset.
  const tokens = computed<SwapTokenOption[]>(() => {
    const byAddress = new Map<string, SwapTokenOption>()

    for (const market of Object.values(marketsStore.state.markets)) {
      const marketState = market?.marketState
      const pools = marketState?.pools_data ?? []
      const oraclePriceDecimals = marketState?.oracle_price_decimals ?? 0
      for (const pd of pools) {
        const address = pd?.pool?.token_address
        if (!address || byAddress.has(address)) {
          continue
        }
        const rawSymbol = pd.pool.token_symbol ?? ''
        const isNative = rawSymbol === 'native'
        const displaySymbol = isNative ? 'XLM' : rawSymbol
        const meta = getFullTokenData(rawSymbol)
        const [, assetIssuer] = destructurePoolAsset(pd.pool.name ?? '')

        const oracleRaw = pd.oracle_asset_price
        const price = oracleRaw && oraclePriceDecimals
          ? Number(bigintToNumber(oracleRaw, oraclePriceDecimals))
          : 0

        byAddress.set(address, {
          symbol: displaySymbol,
          name: meta?.name || displaySymbol,
          icon: meta?.icon || '',
          tokenAddress: address,
          tokenDecimals: pd.pool.token_decimals ?? 7,
          tokenSymbol: rawSymbol,
          isNative,
          assetIssuer: isNative ? undefined : assetIssuer,
          price,
        })
      }
    }

    return [...byAddress.values()].toSorted((a, b) => a.symbol.localeCompare(b.symbol))
  })

  // Pin the swap-submitter market deterministically. `Object.values()` order is
  // insertion-order in practice, but markets can repopulate in a different
  // order across network switches/reloads, which would churn `swapClient`'s
  // identity and re-trigger every downstream effect. Sort by market key so the
  // chosen client is stable across reloads.
  const swapClient = computed<AlulaClient | undefined>(() => {
    const entries = Object.entries(marketsStore.state.markets)
      .toSorted(([a], [b]) => a.localeCompare(b))
    return entries[0]?.[1]?.client
  })

  const fromToken = ref<SwapTokenOption | undefined>()
  const toToken = ref<SwapTokenOption | undefined>()
  // `<input-widget>` emits strings; keep the source of truth as a string and
  // expose a numeric view for the rest of the composable / page.
  const amount = ref<string>('')
  const amountNumber = computed(() => {
    const n = Number(amount.value)
    return Number.isFinite(n) && n > 0 ? n : 0
  })
  const slippage = ref(DEFAULT_SLIPPAGE_PERCENT)

  const swapProviderAddress = computed({
    get: () => multiplyStore.swapProviderAddress,
    set: (value: string) => { multiplyStore.swapProviderAddress = value },
  })

  // Initialize defaults: first two tokens, distinct.
  watch(tokens, (list) => {
    if (!fromToken.value && list[0]) {
      fromToken.value = list.find(t => t.symbol === DEFAULT_FROM_TOKEN_SYMBOL) ?? list[0]
    }
    if (!toToken.value) {
      toToken.value = list.find(t => t.symbol === DEFAULT_TO_TOKEN_SYMBOL) ?? list.find(t => t.tokenAddress !== fromToken.value?.tokenAddress)
    }
  }, { immediate: true })

  // Wallet balance for either side. The XLM reserve carve-out only applies to
  // the From-token (the asset the user is *paying* — they can't spend the
  // network reserve). For the To-token we want to show the user's actual
  // wallet balance: subtracting the reserve from a token they're about to
  // *receive more of* would understate it.
  function walletBalance(token: SwapTokenOption | undefined): number {
    if (!token) {
      return 0
    }
    if (token.isNative) {
      return nativeBalance.value
    }
    return getAssetBalance(token.assetIssuer)
  }
  const fromBalance = computed(() => walletBalance(fromToken.value))
  const toBalance = computed(() => walletBalance(toToken.value))

  // All quotable routes for the current input, ranked best-first by the SDK.
  const routes = ref<SwapRoute[]>([])
  const routesRequest = useLatestRequest<SwapRoute[]>()
  // Tracks whether the in-flight quote was started by the periodic refresh
  // (true) or by direct user input (false). The public `loading` computed
  // below uses this to *hide* the loading state during background refreshes,
  // so the Swap button doesn't flash "Quoting…" every 15 s and the receive
  // panel doesn't briefly drop to "…" while the user is just looking at it.
  const silentRefreshActive = ref(false)
  // User's manual route pin. `undefined` = auto-pick (use the top-ranked route).
  // The pin is by `key` so it survives quote refreshes that produce new SwapRoute
  // object identities for the same provider+path.
  const pinnedRouteKey = ref<string | undefined>()
  const submitting = ref(false)

  const selectedRoute = computed<SwapRoute | undefined>(() => {
    if (pinnedRouteKey.value) {
      const pinned = routes.value.find(r => r.key === pinnedRouteKey.value)
      if (pinned) {
        return pinned
      }
      // Pin no longer matches any quoted route — fall through to the top route.
      // The pin itself is cleared by the watcher below so this getter stays pure
      // (no mutation inside `computed`, which Vue rightly flags as an anti-pattern).
    }
    return routes.value[0]
  })

  // Drop a stale pin once a fresh quote comes back without it. Done here so
  // `selectedRoute` can stay a pure getter, and so the pin clears exactly once
  // per quote rather than on every read.
  watch(routes, (next) => {
    if (pinnedRouteKey.value && !next.some(r => r.key === pinnedRouteKey.value)) {
      pinnedRouteKey.value = undefined
    }
  })

  // `preview` is now derived: take the selected route, layer slippage on top.
  // No extra RPC; switching routes is instant.
  const preview = computed(() => {
    if (!selectedRoute.value || !swapClient.value) {
      return
    }
    return swapClient.value.swap.previewFromRoute(selectedRoute.value, Number(slippage.value))
  })

  // Auto-quote on every input change. Race-token pattern (`useLatestRequest`)
  // drops stale router responses if the user keeps typing. Note: `slippage` is
  // intentionally NOT in the watch source — slippage is applied client-side in
  // `previewFromRoute`, so changing it doesn't require a fresh quote.
  // The debounced wrapper always invokes `quote()` with no args (= non-silent);
  // the silent variant is only used by the periodic refresh below.
  const debouncedQuote = useDebounceFn(quote, QUOTE_DEBOUNCE_MS)
  watch([fromToken, toToken, amount], () => {
    void debouncedQuote()
  }, { deep: false })

  // Background staleness guard: AMM reserves drift, so a quote that's been
  // sitting on screen for a while can revert at submit time. We re-quote on a
  // 15s cadence whenever the input is valid AND the tab is foregrounded — no
  // sense burning RPC for a tab the user can't see. Submitting also pauses the
  // refresh so a stale `routes.value` doesn't replace the route we're signing.
  const visibility = useDocumentVisibility()
  useIntervalFn(() => {
    if (visibility.value !== 'visible') {
      return
    }
    if (submitting.value || routesRequest.loading.value) {
      return
    }
    if (!swapClient.value || !fromToken.value || !toToken.value) {
      return
    }
    if (amountNumber.value <= 0) {
      return
    }
    void quote({ silent: true })
  }, QUOTE_REFRESH_INTERVAL_MS)

  async function quote({ silent = false }: { silent?: boolean } = {}) {
    // User-driven quotes blank the visible state immediately so the user sees
    // that we're working on their new input. Background refreshes keep the
    // previous good state visible until the new one comes back — otherwise the
    // UI jumps every 15 s for no user-perceptible reason.
    if (!silent) {
      routes.value = []
      // Clear any prior error optimistically — the user is acting on the input,
      // so a stale red banner from the last failed quote shouldn't linger
      // through the new fetch.
      routesRequest.error.value = ''
      routesRequest.cancel()
    }

    const amountSnap = amountNumber.value
    if (!swapClient.value || !fromToken.value || !toToken.value || amountSnap <= 0) {
      devLog('%c[Swap routes skipped]', 'color: #888', {
        hasClient: !!swapClient.value,
        from: fromToken.value?.symbol,
        to: toToken.value?.symbol,
        amount: amountSnap,
        silent,
      })
      return
    }
    if (fromToken.value.tokenAddress === toToken.value.tokenAddress) {
      if (!silent) {
        routesRequest.error.value = 'Pick two different tokens'
      }
      return
    }

    const fromSnap = fromToken.value
    const toSnap = toToken.value

    devLog(silent ? '%c[Swap routes ↻]' : '%c[Swap routes ▶]', 'color: #00bfff', {
      from: fromSnap.symbol,
      to: toSnap.symbol,
      amountIn: amountSnap,
    })

    if (silent) {
      silentRefreshActive.value = true
    }
    try {
      await routesRequest.run(
        () => swapClient.value!.swap.getSwapRoutes({
          fromTokenAddress: fromSnap.tokenAddress,
          toTokenAddress: toSnap.tokenAddress,
          fromTokenDecimals: fromSnap.tokenDecimals,
          toTokenDecimals: toSnap.tokenDecimals,
          amountIn: amountSnap,
        }),
        (result) => {
          devLog('%c[Swap routes ✓]', 'color: #2bd17e', result)
          // Treat an empty result during a silent refresh as transient — keep
          // the last-good routes visible rather than wiping them. The user can
          // still try to swap against a slightly stale quote; the next refresh
          // (or their next keystroke) will recover.
          if (result.length === 0) {
            if (!silent) {
              routesRequest.error.value = 'No routes found for this pair'
              routes.value = result
            }
            return
          }
          routes.value = result
          // Keep pin in sync with multiply's stored provider so the rest of the
          // app sees a coherent "current provider", even though the route picker
          // owns the choice on this page.
          if (selectedRoute.value) {
            swapProviderAddress.value = selectedRoute.value.providerAddress
          }
        },
        (e) => {
          devWarn('%c[Swap routes ✗]', 'color: #ff5c5c', e)
          // Silent refresh failures don't surface and — importantly — also
          // don't *clear* the user's prior error. The user is staring at valid
          // prior data with whatever banner the last user-driven quote left
          // behind; the next refresh (or input change) will retry.
        },
      )
    } finally {
      if (silent) {
        silentRefreshActive.value = false
      }
    }
  }

  function pinRoute(key: string | undefined) {
    pinnedRouteKey.value = key
    if (selectedRoute.value) {
      swapProviderAddress.value = selectedRoute.value.providerAddress
    }
  }

  function flip() {
    const tmp = fromToken.value
    fromToken.value = toToken.value
    toToken.value = tmp
  }

  async function submit() {
    if (submitting.value) {
      return
    }
    if (!publicKey.value) {
      throw new Error('Connect a wallet first')
    }
    if (!swapClient.value || !fromToken.value || !toToken.value || amountNumber.value <= 0) {
      return
    }
    // If the user clicks Swap before the 250ms debounce fires (or before the
    // periodic refresh has produced its first quote), force a fresh quote
    // synchronously so we sign against current numbers instead of bailing with
    // a confusing dead-click. Use the silent variant so we don't blank
    // `routes.value` (which would collapse an open route picker mid-click) and
    // so we don't blank a prior error banner. We re-read `selectedRoute`
    // afterwards because `quote()` may rewrite `routes.value` on success.
    let route = selectedRoute.value
    if (!route) {
      await quote({ silent: true })
      route = selectedRoute.value
      if (!route) {
        toast.create({
          title: 'Swap',
          body: 'Could not get a quote for this pair. Try again or pick a different amount.',
          variant: 'danger',
          modelValue: 6000,
        })
        return
      }
    }

    const oblKey = buildObligationKey({ pablicKey: publicKey.value })
    const fromSnap = fromToken.value
    const toSnap = toToken.value
    const amountSnap = amountNumber.value
    const slippageSnap = Number(slippage.value)
    const providerSnap = route.providerAddress
    const pathSnap = route.path

    submitting.value = true
    const info = await toast.create({
      title: 'Swap',
      body: `Sending swap ${fromSnap.symbol} → ${toSnap.symbol} via ${route.providerName}`,
      modelValue: 30_000,
      variant: 'info',
      noProgress: false,
    })

    try {
      const res = await withTimeoutAbort(
        swapClient.value.swap.executeSwap({
          user: oblKey,
          fromTokenAddress: fromSnap.tokenAddress,
          toTokenAddress: toSnap.tokenAddress,
          fromTokenDecimals: fromSnap.tokenDecimals,
          toTokenDecimals: toSnap.tokenDecimals,
          amountIn: amountSnap,
          slippagePercent: slippageSnap,
          swapProviderAddress: providerSnap,
          path: pathSnap,
        }, kit.value),
        TRANSACTION_TIMEOUT,
      )

      toast.create({
        title: 'Swap Success',
        body: 'Transaction sent successfully',
        modelValue: 10_000,
      })
      // Reset the input so the user doesn't accidentally double-submit.
      amount.value = ''
      routes.value = []
      await wallet.loadBalances()
      return res
    } catch (error: any) {
      if (!String(error?.message ?? '').includes('rejected')) {
        toast.create({
          title: 'Swap Error',
          body: String(error?.message || error),
          variant: 'danger',
          modelValue: 10_000,
        })
      }
      throw error
    } finally {
      submitting.value = false
      info?.dismiss()
    }
  }

  return {
    tokens,
    fromToken,
    toToken,
    fromBalance,
    toBalance,
    amount,
    amountNumber,
    slippage,
    routes,
    selectedRoute,
    // Read-only view of the pin. Consumers must mutate via `pinRoute(key)` so
    // the multiply store's `swapProviderAddress` stays in sync — writing
    // `pinnedRouteKey.value` directly would skip that side-effect.
    pinnedRouteKey: readonly(pinnedRouteKey),
    pinRoute,
    isAutoRoute: computed(() => pinnedRouteKey.value == null),
    preview,
    publicKey,
    // Hide background-refresh in-flight state from the UI: a periodic 15s
    // re-quote should never make the Swap button flash "Quoting…" or the
    // receive panel drop to "…" while the user is just looking at the page.
    // User-driven quotes (from input change or submit-flush) leave
    // `silentRefreshActive` false and surface normally.
    loading: computed(() => routesRequest.loading.value && !silentRefreshActive.value),
    error: routesRequest.error,
    submitting,
    isReady: computed(() => !!swapClient.value && tokens.value.length >= 2),
    flip,
    quote,
    submit,
  }
}
