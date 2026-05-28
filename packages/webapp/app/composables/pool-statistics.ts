import type { ApiHistoryData, PoolHistoryBucket } from '~/services'
import { bpsToNumber } from '@alula/client-sdk'
import { chartDateHM, normalizeChartDate } from '~/utils/chart'

const STATISTICS_DATE = [
  {
    label: '1 Day',
    value: 1,
  },
  {
    label: '7 Days',
    value: 7,
  },
  {
    label: '1 Month',
    value: 31,
  },
  {
    label: '1 Year',
    value: 360,
  },
]

export function usePoolStatistics(params: StatisticsComposableParams) {
  const statisticStore = useMarketStatisticsStore()

  const chartFilter = useChartFilter(STATISTICS_DATE, 2)
  const activeFilter = toRef(chartFilter, 'activeFilter')

  const onlyPairAsset = computed(() => !!params.onlyPairAsset)

  const pool = computed(() => statisticStore.state.pool)
  const pairPool = computed(() => statisticStore.state.pairPool)

  const marketAddress = computed(() => statisticStore.marketAddress)
  const poolAddress = computed(() => statisticStore.poolAddress)
  const pairPoolAddress = computed(() => statisticStore.pairPoolAddress)

  const symbol = computed(() => formatAssetSymbol(pool.value?.symbol))
  const decimals = computed(() => pool.value?.decimals ?? 7)
  const pairSymbol = computed(() => formatAssetSymbol(pairPool.value?.symbol))
  const pairDecimals = computed(() => pairPool.value?.decimals ?? 7)
  const hasPairPool = computed(() => Boolean(pairPool.value && pairPoolAddress.value))

  const currencyOptions = computed(() => {
    return ['USD', 'Asset']
  })
  const currency = ref<string>(currencyOptions.value[0] ?? 'USD')

  watch(currencyOptions, (options) => {
    if (!options.includes(currency.value)) {
      currency.value = options[0] ?? 'USD'
    }
  }, { immediate: true })

  const selectedBucket = computed(() => bucketByFilterValue(Number(activeFilter.value.value) || 31))

  const historyData = computed(() => statisticStore.historyMap.get(`${marketAddress.value}-${poolAddress.value}-${selectedBucket.value}`) ?? [])
  const pairHistoryData = computed(() => {
    if (!pairPoolAddress.value) {
      return []
    }

    return statisticStore.historyMap.get(`${marketAddress.value}-${pairPoolAddress.value}-${selectedBucket.value}`) ?? []
  })

  const currentHistoryData = computed(() => historyData.value.at(-1))
  const currentPairHistoryData = computed(() => pairHistoryData.value.at(-1))
  const currentChartTypeData = computed(() => currentHistoryData.value?.[params.chartType] ?? 0)
  const currentPairChartTypeData = computed(() => currentPairHistoryData.value?.[params.chartType] ?? 0)
  const currentPrice = computed(() => Number(currentHistoryData.value?.oracle_price_usd ?? 0))
  const currentPairPrice = computed(() => Number(currentPairHistoryData.value?.oracle_price_usd ?? 0))

  const cardLabel = computed(() => {
    switch (params.chartType) {
      case 'total_supplied':
        return 'Total supplied'
      case 'total_borrowed':
        return 'Total borrowed'
      case 'supply_apy_bps':
        return 'Supply APY'
      case 'borrow_apy_bps':
        return 'Borrow APY'
      case 'tvl_usd_cents':
        return 'TVL'
      case 'utilization_bps':
        return 'Utilization'
      case 'oracle_price_usd':
        return 'Price'
      default: return 'Statistic'
    }
  })

  const cardValue = computed(() => {
    const chartData = onlyPairAsset.value ? currentPairChartTypeData.value : currentChartTypeData.value
    const price = onlyPairAsset.value ? currentPairPrice.value : currentPrice.value
    switch (params.chartType) {
      case 'total_supplied': {
        const supplyNum = bigintToNumber(BigInt(Number(chartData)), decimals.value) || 0
        const supplyUSD = Number(supplyNum) * price
        return {
          raw: supplyUSD,
          formatted: `$${shortenNumber(supplyUSD, 2, 2)}`,
        }
      }
      case 'total_borrowed': {
        const borrowNum = bigintToNumber(BigInt(Number(chartData)), decimals.value) || 0
        const borrowUSD = Number(borrowNum) * price
        return {
          raw: borrowUSD,
          formatted: `$${shortenNumber(borrowUSD, 2, 2)}`,
        }
      }
      case 'supply_apy_bps':{
        const apy = Number(chartData) / 100
        return {
          raw: apy,
          formatted: `${truncatePercent(apy, 2)}%`,
        }
      }
      case 'borrow_apy_bps':{
        const apy = Number(chartData) / 100
        return {
          raw: apy,
          formatted: `${truncatePercent(apy, 2)}%`,
        }
      }
      case 'tvl_usd_cents': {
        const tvl = Number(chartData) / 100
        return {
          raw: tvl,
          formatted: `$${shortenNumber(tvl, 2, 2)}`,
        }
      }

      case 'utilization_bps': {
        const utilization = bpsToNumber(Number(chartData))
        return {
          raw: utilization,
          formatted: `${truncatePercent(utilization, 2)}%`,
        }
      }

      case 'oracle_price_usd': {
        const num = Number(chartData) || 0
        const price = num < 0.01 ? '<0.01' : shortenNumber(num, 2, 2)
        return {
          raw: num,
          formatted: `$${price}`,
        }
      }

      default: return { raw: 0, formatted: 0 }
    }
  })

  const chartPoints = computed(() => {
    const filterVal = Number(activeFilter.value.value)
    const length = filterVal > 1 ? filterVal : historyData.value.length
    return buildChartPoints(historyData.value, length, params.chartType, decimals.value, currency.value, filterVal)
  })

  const pairChartPoints = computed(() => {
    const filterVal = Number(activeFilter.value.value)
    const length = filterVal > 1 ? filterVal : pairHistoryData.value.length
    return buildChartPoints(pairHistoryData.value, length, params.chartType, pairDecimals.value, currency.value, filterVal)
  })

  watch(activeFilter, async (f) => {
    const bucket = bucketByFilterValue(Number(f.value))
    if (!statisticStore.marketAddress || !statisticStore.poolAddress) {
      return
    }

    await statisticStore.getPoolHistoryData(statisticStore.marketAddress, statisticStore.poolAddress, bucket)

    if (statisticStore.pairPoolAddress) {
      await statisticStore.getPoolHistoryData(statisticStore.marketAddress, statisticStore.pairPoolAddress, bucket)
    }
  })

  return {
    cardLabel,
    cardValue,
    activeFilter,
    chartFilter,
    historyData,
    chartPoints,
    symbol,
    pairChartPoints,
    pairSymbol,
    hasPairPool,

    currencyOptions,
    currency,
  }
}

type StatisticsComposableParams = {
  chartType: keyof ApiHistoryData
  onlyMarketAsset?: boolean
  onlyPairAsset?: boolean
}

function bucketByFilterValue(val: number): PoolHistoryBucket {
  switch (val) {
    case 1: return '15m'
    case 7: return '1d'
    case 31: return '1d'
    case 360: return '1d'
    default: return '1d'
  }
}

function buildChartPoints(
  history: ApiHistoryData[],
  length: number,
  chartType: keyof ApiHistoryData,
  decimals: number,
  currency: string,
  filterValue: number,
) {
  return [...history].slice(0, length).map((d) => {
    const rawVal = Number(d[chartType])
    let value: number

    switch (chartType) {
      case 'total_supplied':
      case 'total_borrowed': {
        const price = Number(d.oracle_price_usd ?? 0)
        const num = bigintToNumber(BigInt(rawVal), decimals) || 0
        value = currency === 'USD' ? Number(num) * price : Number(num)
        break
      }
      case 'supply_apy_bps':
      case 'borrow_apy_bps':
        value = rawVal / 100
        break
      case 'tvl_usd_cents':
        value = rawVal / 100
        break
      case 'utilization_bps':
        value = bpsToNumber(rawVal)
        break
      default:
        value = rawVal
    }

    return {
      label: filterValue === 1 ? chartDateHM(String(d.start_time)) : normalizeChartDate(String(d.start_time), false),
      date: String(d.start_time),
      value,
    }
  })
}

function formatAssetSymbol(symbol?: string) {
  if (!symbol) {
    return ''
  }

  return symbol === 'native' ? 'XLM' : symbol
}
