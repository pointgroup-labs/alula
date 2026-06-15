import type { PoolData } from '@alula/market-sdk'
import type { MaybeRefOrGetter } from 'vue'
import { Buffer } from 'node:buffer'
import { computed, toValue } from 'vue'

const SECONDS_PER_YEAR = 31_556_926
const SECONDS_PER_WEEK = 604_800

/* TODO: remove after test and use real price */
const TEST_AQUA_PRICE = 0.000_377_8

type FarmType = 'supply' | 'borrow'

type UseFarmsParams = {
  marketName: MaybeRefOrGetter<string>
  pool: MaybeRefOrGetter<PoolData>
  farmType: MaybeRefOrGetter<FarmType>
}

export function useFarms({
  marketName,
  pool,
  farmType,
}: UseFarmsParams) {
  const marketsStore = useMarketsStore()

  const { getTokenByAddress } = useTokensStore()
  const { getMarketFarms } = useFarmsStore()

  const nowUnix = ref(Date.now() / 1000)

  const isSupplyFarm = computed(() => toValue(farmType) === 'supply')

  const poolData = computed(() => toValue(pool))

  const marketFarms = computed(() => {
    return getMarketFarms(toValue(marketName))
  })

  const poolFarmId = computed(() => {
    return isSupplyFarm.value
      ? poolData.value.pool?.farm_supply
      : poolData.value.pool?.farm_debt
  })

  const poolFarm = computed(() => {
    const farmId = poolFarmId.value

    if (!farmId || !marketFarms.value) {
      return null
    }

    const farmIdHex = Buffer.isBuffer(farmId)
      ? farmId.toString('hex')
      : Buffer.from(farmId).toString('hex')

    return marketFarms.value.find((farm) => {
      return Buffer.from(farm.farm.id).toString('hex') === farmIdHex
    }) ?? null
  })

  const actualRewards = computed(() => {
    return poolFarm.value?.rewards?.filter((reward) => {
      const points = reward.reward_schedule_curve.points

      const start = Number(points.at(0)?.ts_start) || 0
      const end = Number(points.at(-1)?.ts_start) || 0

      return nowUnix.value > start && nowUnix.value < end
    }) ?? []
  })

  const preparedRewards = computed(() => {
    const currentPool = poolData.value

    return actualRewards.value.map((reward) => {
      const asset = getTokenByAddress(reward.reward_token)

      const rewardAssetDecimals = asset?.decimals ?? 7

      const rewardPerTimeUnit = Number(
        reward.reward_schedule_curve.points.at(0)?.reward_per_time_unit,
      ) || 0

      const rewardPerSec = rewardPerTimeUnit / 10 ** rewardAssetDecimals
      const rewardPerYear = rewardPerSec * SECONDS_PER_YEAR

      const rewardAssetPrice = getAssetPrice(reward.reward_token)
      const rewardPerYearUSD = rewardPerYear * rewardAssetPrice

      const rewardPerWeek = rewardPerSec * SECONDS_PER_WEEK
      const rewardPerWeekUSD = rewardPerWeek * rewardAssetPrice

      const poolAssetAmount = isSupplyFarm.value
        ? Number(bigintToNumber(currentPool.total_supply, currentPool.pool.token_decimals)) || 0
        : Number(bigintToNumber(currentPool.pool.total_borrowed, currentPool.pool.token_decimals)) || 0

      const poolAssetPrice = getAssetPrice(currentPool.pool.pool_address)
      const poolAssetAmountUSD = poolAssetAmount * poolAssetPrice

      const rewardAPY = poolAssetAmountUSD > 0
        ? rewardPerYearUSD / poolAssetAmountUSD * 100
        : 0

      return {
        asset,
        rewardToken: reward.reward_token,
        rewardAPY,
        rewardPerWeekUSD,
      }
    })
  })

  const apyData = computed(() => {
    const currentPool = poolData.value
    const lendApyBps = isSupplyFarm.value
      ? currentPool.apy.supply_bps
      : currentPool.apy.borrow_bps

    const lendAPY = lendApyBps / 100

    const totalRewardsAPY = preparedRewards.value.reduce((acc, reward) => {
      return acc + reward.rewardAPY
    }, 0)

    const combinedAPY = isSupplyFarm.value
      ? lendAPY + totalRewardsAPY
      : lendAPY - totalRewardsAPY

    return {
      lendAPY,
      combinedAPY,
    }
  })

  const assetPriceByAddress = computed(() => {
    const map = new Map<string, number>()

    for (const marketName in marketsStore.state.markets) {
      const market = marketsStore.state.markets[marketName]
      const oraclePriceDecimals = market?.marketState.oracle_price_decimals ?? 14

      for (const pool of market?.marketState?.pools_data ?? []) {
        map.set(
          pool.pool.pool_address,
          Number(bigintToNumber(pool.oracle_asset_price, oraclePriceDecimals)) || 0,
        )
      }
    }

    return map
  })

  function getAssetPrice(address: string) {
    return assetPriceByAddress.value.get(address) ?? TEST_AQUA_PRICE
  }

  let interval: any

  onMounted(() => {
    interval = setInterval(() => {
      nowUnix.value = Date.now() / 1000
    }, 60_000)
  })

  onUnmounted(() => {
    clearInterval(interval)
  })

  return {
    poolFarmId,
    marketFarms,
    poolFarm,
    actualRewards,
    preparedRewards,

    apyData,

    getAssetPrice,
  }
}
