import type { PoolData } from '@alula/market-sdk'
import type { MaybeRefOrGetter } from 'vue'
import { Buffer } from 'node:buffer'
import { computed, onMounted, onUnmounted, ref, toValue } from 'vue'

const SECONDS_PER_YEAR = 31_556_926
const SECONDS_PER_WEEK = 604_800

type UseFarmsParams = {
  marketName: MaybeRefOrGetter<string | undefined | null>
  pool: MaybeRefOrGetter<PoolData | undefined | null>
}

export function useFarms({
  marketName,
  pool,
}: UseFarmsParams) {
  const { getAssetPrice } = usePriceStore()
  const { getTokenByAddress } = useTokensStore()
  const { getMarketFarms } = useFarmsStore()

  const nowUnix = ref(Date.now() / 1000)

  const poolData = computed(() => toValue(pool))

  const marketFarms = computed(() => {
    const name = toValue(marketName)

    if (!name) {
      return null
    }

    return getMarketFarms(name)
  })

  const supplyFarmId = computed(() => {
    return poolData.value?.pool?.farm_supply
  })

  const borrowFarmId = computed(() => {
    return poolData.value?.pool?.farm_debt
  })

  const supplyFarm = computed(() => {
    return findFarmById(supplyFarmId.value)
  })

  const borrowFarm = computed(() => {
    return findFarmById(borrowFarmId.value)
  })

  function getActualRewards(farm: typeof supplyFarm.value) {
    return farm?.rewards?.filter((reward) => {
      const points = reward.reward_schedule_curve.points

      const start = Number(points.at(0)?.ts_start) || 0
      const end = Number(points.at(-1)?.ts_start) || 0

      return nowUnix.value > start && nowUnix.value < end
    }) ?? []
  }

  const supplyActualRewards = computed(() => {
    return getActualRewards(supplyFarm.value)
  })

  const borrowActualRewards = computed(() => {
    return getActualRewards(borrowFarm.value)
  })

  const supplyPreparedRewards = computed(() => {
    return prepareRewards(supplyActualRewards.value, 'supply')
  })

  const borrowPreparedRewards = computed(() => {
    return prepareRewards(borrowActualRewards.value, 'borrow')
  })

  const supplyApyData = computed(() => {
    return getApyData(supplyPreparedRewards.value, 'supply')
  })

  const borrowApyData = computed(() => {
    return getApyData(borrowPreparedRewards.value, 'borrow')
  })

  const isHaveFarms = computed(() => {
    const farms = [...supplyActualRewards.value, ...borrowActualRewards.value].flat()
    return farms.some(f => Number(f.rewards_available) > 0)
  })

  function findFarmById(farmId: any) {
    if (!farmId || !marketFarms.value) {
      return null
    }

    const farmIdHex = Buffer.isBuffer(farmId)
      ? farmId.toString('hex')
      : Buffer.from(farmId).toString('hex')

    return marketFarms.value.find((farm) => {
      return Buffer.from(farm.farm.id).toString('hex') === farmIdHex
    }) ?? null
  }

  function getApyData(
    preparedRewards: typeof supplyPreparedRewards.value,
    type: 'supply' | 'borrow',
  ) {
    const currentPool = poolData.value

    if (!currentPool) {
      return {
        lendAPY: 0,
        rewardsAPY: 0,
        combinedAPY: 0,
      }
    }

    const lendApyBps = type === 'supply'
      ? currentPool.apy.supply_bps
      : currentPool.apy.borrow_bps

    const lendAPY = lendApyBps / 100

    const rewardsAPY = preparedRewards.reduce((acc, reward) => {
      return acc + reward.rewardAPY
    }, 0)

    const combinedAPY = type === 'supply'
      ? lendAPY + rewardsAPY
      : lendAPY - rewardsAPY

    return {
      lendAPY,
      rewardsAPY,
      combinedAPY,
    }
  }

  function prepareRewards(
    rewards: typeof supplyActualRewards.value,
    type: 'supply' | 'borrow',
  ) {
    const currentPool = poolData.value

    if (!currentPool) {
      return []
    }

    return rewards.map((reward) => {
      const asset = getTokenByAddress(reward.reward_token)

      const rewardAssetDecimals = asset?.decimals ?? 7

      const rewardPerTimeUnit = Number(
        reward.reward_schedule_curve.points.at(0)?.reward_per_time_unit,
      ) || 0

      const rewardPerSec = rewardPerTimeUnit / 10 ** rewardAssetDecimals
      const rewardPerYear = rewardPerSec * SECONDS_PER_YEAR

      const rewardAssetPrice = getAssetPrice(asset?.symbol)
      const rewardPerYearUSD = rewardPerYear * rewardAssetPrice

      const rewardPerWeek = rewardPerSec * SECONDS_PER_WEEK
      const rewardPerWeekUSD = rewardPerWeek * rewardAssetPrice

      const poolAssetAmount = type === 'supply'
        ? Number(bigintToNumber(currentPool.total_supply, currentPool.pool.token_decimals)) || 0
        : Number(bigintToNumber(currentPool.pool.total_borrowed, currentPool.pool.token_decimals)) || 0

      const poolAssetPrice = getAssetPrice(currentPool.pool.token_symbol)
      const poolAssetAmountUSD = poolAssetAmount * poolAssetPrice

      const rewardAPY = poolAssetAmountUSD > 0
        ? rewardPerYearUSD / poolAssetAmountUSD * 100
        : 0

      return {
        asset,
        rewardToken: reward.reward_token,
        rewardAPY,
        rewardPerWeek,
        rewardPerWeekUSD,
        available: reward.rewards_available,
      }
    })
  }

  let interval: ReturnType<typeof setInterval>

  onMounted(() => {
    interval = setInterval(() => {
      nowUnix.value = Date.now() / 1000
    }, 60_000)
  })

  onUnmounted(() => {
    clearInterval(interval)
  })

  return {
    isHaveFarms,

    marketFarms,

    supplyFarmId,
    borrowFarmId,

    supplyFarm,
    borrowFarm,

    supplyActualRewards,
    borrowActualRewards,

    supplyPreparedRewards,
    borrowPreparedRewards,

    supplyApyData,
    borrowApyData,

    getAssetPrice,
  }
}
