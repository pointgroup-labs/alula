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
  const priceStore = usePriceStore()

  const chartFilter = useChartFilter(STATISTICS_DATE, 2)
  const activeFilter = toRef(chartFilter, 'activeFilter')

  const pool = computed(() => statisticStore.state.pool)

  const marketAddress = computed(() => statisticStore.marketAddress)
  const poolAddress = computed(() => statisticStore.poolAddress)

  const symbol = computed(() => pool.value?.symbol ?? '')
  const decimals = computed(() => pool.value?.decimals ?? 7)
  const price = computed(() => priceStore.assetsPrices[symbol.value] ?? 0)

  const currencyOptions = computed(() => {
    const poolCurrencies = ['USD']
    if (symbol.value) {
      const name = symbol.value === 'native' ? 'XLM' : symbol.value
      poolCurrencies.push(name)
    }
    return poolCurrencies
  })
  const currency = ref(currencyOptions.value[0])

  const selectedBucket = computed(() => bucketByFilterValue(Number(activeFilter.value.value) || 31))

  const historyData = computed(() => statisticStore.historyMap.get(`${marketAddress.value}-${poolAddress.value}-${selectedBucket.value}`) ?? [])
  const lastDateData = computed(() => historyData.value[0]?.[params.chartType] ?? 0)

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
      default: return 'Statistic'
    }
  })

  const cardValue = computed(() => {
    switch (params.chartType) {
      case 'total_supplied': {
        const supplyNum = bigintToNumber(BigInt(Number(lastDateData.value)), decimals.value) || 0
        const supplyUSD = Number(supplyNum) * price.value
        return {
          raw: supplyUSD,
          formatted: `$${shortenNumber(supplyUSD, 0, 0)}`,
        }
      }
      case 'total_borrowed': {
        const borrowNum = bigintToNumber(BigInt(Number(lastDateData.value)), decimals.value) || 0
        const borrowUSD = Number(borrowNum) * price.value
        return {
          raw: borrowUSD,
          formatted: `$${shortenNumber(borrowUSD, 0, 0)}`,
        }
      }
      case 'supply_apy_bps':{
        const apy = Number(lastDateData.value) / 100
        return {
          raw: apy,
          formatted: `${truncatePercent(apy, 2)}%`,
        }
      }
      case 'borrow_apy_bps':{
        const apy = Number(lastDateData.value) / 100
        return {
          raw: apy,
          formatted: `${truncatePercent(apy, 2)}%`,
        }
      }
      case 'tvl_usd_cents': {
        const tvl = Number(lastDateData.value) / 100
        return {
          raw: tvl,
          formatted: `$${shortenNumber(tvl, 0, 0)}`,
        }
      }

      case 'utilization_bps': {
        const utilization = bpsToNumber(Number(lastDateData.value))
        return {
          raw: utilization,
          formatted: `${truncatePercent(utilization, 2)}%`,
        }
      }

      default: return { raw: 0, formatted: 0 }
    }
  })

  const chartPoints = computed(() => {
    const filterVal = Number(activeFilter.value.value)
    const length = filterVal > 1 ? filterVal : historyData.value.length
    return [...historyData.value]?.slice(0, length)?.map((d) => {
      const rawVal = Number(d[params.chartType])
      let value: number
      switch (params.chartType) {
        case 'total_supplied':
        case 'total_borrowed': {
          const num = bigintToNumber(BigInt(rawVal), decimals.value) || 0
          value = currency.value === 'USD' ? Number(num) * price.value : Number(num)
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
        label: activeFilter.value.value === 1 ? chartDateHM(String(d.start_time)) : normalizeChartDate(String(d.start_time), false),
        date: String(d.start_time),
        value,
      }
    })
  })

  watch(activeFilter, async (f) => {
    const bucket = bucketByFilterValue(Number(f.value))
    await statisticStore.getPoolHistoryData(statisticStore.marketAddress, statisticStore.poolAddress, bucket)
  })

  return {
    cardLabel,
    cardValue,
    activeFilter,
    chartFilter,
    historyData,
    chartPoints,

    currencyOptions,
    currency,
  }
}

type StatisticsComposableParams = {
  chartType: keyof ApiHistoryData
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
