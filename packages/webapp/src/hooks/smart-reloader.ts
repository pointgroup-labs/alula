type Job = () => Promise<any>

export function useSmartReloader() {
  const marketsStore = useMarketsStore()
  const userStore = useUserStore()

  const POOL_JOBS = ref<Job[]>([])
  const OBLIGATION_JOBS = ref<Job[]>([])

  const POOL_EVERY_MS = 30_000
  const OBLIGATION_EVERY_MS = 120_000
  const COUNTDOWN_TICK_MS = 1000
  const POOL_EVERY_S = POOL_EVERY_MS / 1000
  const OBLIGATION_EVERY_S = OBLIGATION_EVERY_MS / 1000

  let poolIntervalId: NodeJS.Timeout | null = null
  let obligationIntervalId: NodeJS.Timeout | null = null
  let poolCountdownId: NodeJS.Timeout | null = null
  let obligationCountdownId: NodeJS.Timeout | null = null

  const poolCountdown = ref(POOL_EVERY_S)
  const obligationCountdown = ref(OBLIGATION_EVERY_S)

  // Watch only the structural fingerprint (market names + pool addresses).
  // Pool *data* updates (prices, APY, etc.) also mutate state.markets deeply,
  // which would retrigger a full deep watch and cause a feedback loop where
  // every updatePool call schedules yet another rebuild of POOL_JOBS.
  // Using a string fingerprint means the callback only fires when markets are
  // added/removed or pool addresses change — not on every data refresh.
  watchDebounced(
    () => Object.entries(marketsStore.state.markets)
      .flatMap(([name, market]) =>
        (market?.marketState?.pools_data ?? [])
          .map(d => d?.pool?.pool_address)
          .filter(Boolean)
          .map(addr => `${name}:${addr}`),
      )
      .toSorted()
      .join('|'),
    () => {
      const markets = marketsStore.state.markets
      const jobs: Job[] = []

      for (const [name, market] of Object.entries(markets)) {
        const client = market?.client
        if (!name || !client) {
          continue
        }

        for (const data of (market?.marketState?.pools_data ?? [])) {
          const addr = data?.pool?.pool_address
          if (!addr) {
            continue
          }
          jobs.push(() => marketsStore.updatePool(addr, name, client, false))
        }
      }
      POOL_JOBS.value = jobs
    },
    { debounce: 400, immediate: true },
  )

  // Same principle for obligations: watch only the set of keys, not deep data.
  watchDebounced(
    () => {
      const oblKeys = Object.keys(userStore.state.obligations ?? {}).toSorted().join(',')
      const multOblKeys = Object.keys(userStore.state.multiplyObligations ?? {}).toSorted().join(',')
      // Also include market multiply_pairs structure since jobs depend on it
      const pairsKey = Object.entries(marketsStore.state.markets)
        .map(([name, m]) =>
          `${name}:${(m?.marketState?.multiply_pairs ?? []).map(p => `${p.deposit_pool}-${p.borrow_pool}`).join(',')}`)
        .toSorted()
        .join('|')
      return `${oblKeys}||${multOblKeys}||${pairsKey}`
    },
    () => {
      const obligations = userStore.state.obligations
      const multiplyObligations = userStore.state.multiplyObligations
      const jobs: Job[] = []
      if (!obligations && !multiplyObligations) {
        return
      }

      for (const key in obligations) {
        const obligation = obligations[key]
        const client = marketsStore.state.markets[key]?.client
        if (!obligation || !client) {
          continue
        }

        jobs.push(() => userStore.updateUserObligation(key, client, false))
      }

      for (const key in multiplyObligations) {
        const obligation = multiplyObligations[key]
        const market = marketsStore.state.markets[key]
        const client = market?.client
        if (!obligation || !client) {
          continue
        }
        for (const p of (market?.marketState.multiply_pairs ?? [])) {
          jobs.push(() => userStore.updateUserMultiplyObligation({
            market: key,
            client,
            depositPoolAddress: p.deposit_pool,
            borrowPoolAddress: p.borrow_pool,
            withLogs: false }))
        }
      }

      OBLIGATION_JOBS.value = jobs
    }, { debounce: 400, immediate: true })

  async function runWithLimit(jobs: Job[], limit = 5, pauseMs = 500) {
    for (let i = 0; i < jobs.length; i += limit) {
      const batch = jobs.slice(i, i + limit).map(j => j())
      await Promise.allSettled(batch)
      if (i + limit < jobs.length) {
        await sleep(pauseMs)
      }
    }
  }

  // Per-queue running flags to prevent concurrent overlapping runs.
  const poolRunGuard = { running: false }
  const obligationRunGuard = { running: false }

  function tick(jobs: Job[], guard: { running: boolean }) {
    if (document.visibilityState === 'hidden' || !navigator.onLine) {
      return
    }
    if (guard.running) {
      return
    }
    guard.running = true
    void runWithLimit(jobs).finally(() => {
      guard.running = false
    })
  }

  function startPool() {
    if (poolIntervalId != null) {
      return
    }
    poolCountdown.value = POOL_EVERY_S
    poolIntervalId = globalThis.setInterval(() => {
      poolCountdown.value = POOL_EVERY_S
      tick(POOL_JOBS.value, poolRunGuard)
    }, POOL_EVERY_MS)
    poolCountdownId = globalThis.setInterval(() => {
      poolCountdown.value = Math.max(0, poolCountdown.value - 1)
    }, COUNTDOWN_TICK_MS)
  }

  function startObligation() {
    if (obligationIntervalId != null) {
      return
    }
    obligationCountdown.value = OBLIGATION_EVERY_S
    obligationIntervalId = globalThis.setInterval(() => {
      obligationCountdown.value = OBLIGATION_EVERY_S
      tick(OBLIGATION_JOBS.value, obligationRunGuard)
    }, OBLIGATION_EVERY_MS)
    obligationCountdownId = globalThis.setInterval(() => {
      obligationCountdown.value = Math.max(0, obligationCountdown.value - 1)
    }, COUNTDOWN_TICK_MS)
  }

  function start() {
    startPool()
    startObligation()
  }

  function stop() {
    if (poolIntervalId != null) {
      clearInterval(poolIntervalId)
      poolIntervalId = null
    }
    if (obligationIntervalId != null) {
      clearInterval(obligationIntervalId)
      obligationIntervalId = null
    }
    if (poolCountdownId != null) {
      clearInterval(poolCountdownId)
      poolCountdownId = null
    }
    if (obligationCountdownId != null) {
      clearInterval(obligationCountdownId)
      obligationCountdownId = null
    }
  }

  onUnmounted(stop)

  return {
    start,
    stop,
    poolCountdown,
    obligationCountdown,
    refreshPools: () => {
      poolCountdown.value = POOL_EVERY_S
      tick(POOL_JOBS.value, poolRunGuard)
    },
    refreshObligations: () => {
      obligationCountdown.value = OBLIGATION_EVERY_S
      tick(OBLIGATION_JOBS.value, obligationRunGuard)
    },
  }
}
