type Job = () => Promise<any>

export function useSmartReloader() {
  const marketsStore = useMarketsStore()
  const userStore = useUserStore()

  const POOL_JOBS = ref<Job[]>([])
  const OBLIGATION_JOBS = ref<Job[]>([])

  const POOL_EVERY_MS = 30_000
  const OBLIGATION_EVERY_MS = 120_000

  let poolIntervalId: NodeJS.Timeout | null = null
  let obligationIntervalId: NodeJS.Timeout | null = null

  watchDebounced(
    () => marketsStore.state.markets,
    (markets) => {
      const jobs: Job[] = []
      if (!markets) {
        return
      }

      for (const market of Object.values<any>(markets)) {
        const name = market?.marketState?.name
        const client = market?.client
        if (!name || !client) {
          continue
        }

        for (const p of (market.pools ?? [])) {
          const addr = p?.pool_address
          if (!addr) {
            continue
          }
          jobs.push(() => marketsStore.updatePool(addr, name, client))
        }

        for (const lp of (market.leveragePools ?? [])) {
          jobs.push(() =>
            marketsStore.updateLeveragePool({
              market: name,
              deposit_pool_address: lp.deposit_pool,
              borrow_pool_address: lp.borrow_pool,
              client,
            }),
          )
        }
      }
      POOL_JOBS.value = jobs
    },
    { debounce: 400, deep: true, immediate: true },
  )

  watchDebounced(
    [() => userStore.state.obligations,
      () => userStore.state.multiplyObligations],
    ([obligations, multiplyObligations]) => {
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

        jobs.push(() => userStore.updateUserObligation(key, client))
      }

      for (const key in multiplyObligations) {
        const obligation = multiplyObligations[key]
        const market = marketsStore.state.markets[key]
        const client = market?.client
        if (!obligation || !client) {
          continue
        }
        for (const p of (market?.leveragePools ?? [])) {
          jobs.push(() => userStore.updateUserMultiplyObligation({
            market: key,
            client,
            depositPoolAddress: p.deposit_pool,
            borrowPoolAddress: p.borrow_pool,
          }))
        }
      }

      OBLIGATION_JOBS.value = jobs
    }, { debounce: 400, deep: true, immediate: true })

  async function runWithLimit(jobs: Job[], limit = 5, pauseMs = 500) {
    for (let i = 0; i < jobs.length; i += limit) {
      const batch = jobs.slice(i, i + limit).map(j => j())
      await Promise.allSettled(batch)
      if (i + limit < jobs.length) {
        await sleep(pauseMs)
      }
    }
  }

  function tick(jobs: Job[]) {
    if (document.visibilityState === 'hidden' || !navigator.onLine) {
      return
    }
    void runWithLimit(jobs)
  }

  function startPool() {
    if (poolIntervalId != null) {
      return
    }
    poolIntervalId = globalThis.setInterval(() => tick(POOL_JOBS.value), POOL_EVERY_MS)
  }

  function startObligation() {
    if (obligationIntervalId != null) {
      return
    }
    obligationIntervalId = globalThis.setInterval(() => tick(OBLIGATION_JOBS.value), OBLIGATION_EVERY_MS)
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
  }

  onUnmounted(stop)

  return {
    start,
    stop,
    refreshPools: () => tick(POOL_JOBS.value),
    refreshObligations: () => tick(OBLIGATION_JOBS.value),
  }
}
