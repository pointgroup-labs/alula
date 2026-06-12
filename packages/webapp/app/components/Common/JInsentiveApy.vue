<script lang="ts" setup>
import type { FarmState } from '@alula/farms-sdk'
import type { PoolData } from '@alula/market-sdk'

const {
  apy,
  variant = 'cyan',
  size = 'sm',
  farms,
  poolData,
  farmType,
} = defineProps<{
  apy: string
  variant?: 'cyan' | 'indigo' | 'success' | 'danger'
  size?: 'lg' | 'md' | 'sm'
  farms?: FarmState[]
  farmType: 'supply' | 'borrow'
  poolData: PoolData
}>()

const SECONDS_PER_YEAR = 31_556_926
const SECONDS_PER_WEEK = 604_800

/* TODO: remove after test and use real price */
const TEST_AQUA_PRICE = 0.000_377_8

const marketsStore = useMarketsStore()
const { getTokenByAddress } = useTokensStore()

const isSupplyFarms = computed(() => farmType === 'supply')

const poolFarmId = computed(() => isSupplyFarms.value ? poolData.pool?.farm_supply : poolData.pool?.farm_debt)
const poolFarm = computed(() => {
  if (!poolFarmId.value || !farms) {
    return null
  }
  const farmIdHex = Buffer.isBuffer(poolFarmId.value) ? poolFarmId.value.toString('hex') : Buffer.from(poolFarmId.value).toString('hex')
  return farms.find(f => Buffer.from(f.farm.id).toString('hex') === farmIdHex) || null
})

const actualRewards = computed(() => {
  const activeRewards = poolFarm.value?.rewards?.filter((r) => {
    const nowUnix = Date.now() / 1000
    const points = r.reward_schedule_curve.points
    const start = Number(points.at(0)?.ts_start) || 0
    const end = Number(points.at(-1)?.ts_start) || 0
    return nowUnix > start && nowUnix < end
  })
  return activeRewards ?? []
})

const preparedRewards = computed(() => {
  const rewards = actualRewards.value.map((r) => {
    /* reward asset data */
    const asset = getTokenByAddress(r.reward_token)

    /* reward data */
    const rewardAssetDecimals = asset?.decimals ?? 7
    const rewardPerTimeUnit = Number(r.reward_schedule_curve.points.at(0)?.reward_per_time_unit) || 0
    const rewardPerSec = Number(rewardPerTimeUnit) / 10 ** rewardAssetDecimals
    const rewardPerYear = rewardPerSec * SECONDS_PER_YEAR
    const rewardAssetPrice = getAssetPrice(r.reward_token)
    const rewardPerYearUSD = rewardPerYear * rewardAssetPrice

    /* reward per week data */
    const rewardPerWeek = rewardPerSec * SECONDS_PER_WEEK
    const rewardPerWeekUSD = rewardPerWeek * rewardAssetPrice

    /* pool data */
    const poolAssetAmount = isSupplyFarms.value
      ? Number(bigintToNumber(poolData.total_supply, poolData.pool.token_decimals)) || 0
      : Number(bigintToNumber(poolData.pool.total_borrowed, poolData.pool.token_decimals)) || 0

    const poolAssetPrice = getAssetPrice(poolData.pool.pool_address)
    const poolAssetAmountUSD = poolAssetAmount * poolAssetPrice

    /* reward APY data */
    const rewardAPY = poolAssetAmountUSD > 0 ? (rewardPerYearUSD / poolAssetAmountUSD) * 100 : 0
    return {
      asset,
      rewardToken: r.reward_token,
      rewardAPY,
      rewardPerWeekUSD,
    }
  })
  return rewards
})

const apyData = computed(() => {
  const lendApyBps = isSupplyFarms.value ? poolData.apy.supply_bps : poolData.apy.borrow_bps
  const lendAPY = lendApyBps / 100
  const totalRewardsAPY = preparedRewards.value.reduce((acc, el) => acc += el.rewardAPY, 0)
  const combinedAPY = isSupplyFarms.value
    ? lendAPY + totalRewardsAPY
    : lendAPY - totalRewardsAPY
  return {
    lendAPY,
    combinedAPY,
  }
})

const farmsClasses = computed(() => {
  const classes = []
  if (poolFarm.value) {
    classes.push(`farms-badge--${farmType}`)
  }
  return classes
})

function getAssetPrice(address: string) {
  for (const marketName in marketsStore.state.markets) {
    const market = marketsStore.state.markets[marketName]
    const actualPool = market?.marketState?.pools_data?.find((p) => {
      return p.pool.pool_address === address
    })
    if (actualPool) {
      const oraclePriceDecimals = market?.marketState.oracle_price_decimals ?? 14
      const oracleAssetPrice = actualPool.oracle_asset_price
      return Number(bigintToNumber(oracleAssetPrice, oraclePriceDecimals)) || 0
    }
  }
  /* TODO: remove after we have real price for not exist in market asset */
  return TEST_AQUA_PRICE
}
</script>

<template>
  <j-pill-label
    :variant="variant"
    :size="size"
    :class="farmsClasses"
  >
    <template v-if="actualRewards.length > 0">
      <j-tooltip content-class="farms-info-tip">
        {{ truncatePercent(apyData.combinedAPY, 2) }}% <i-app-lighting-icon />

        <template #content>
          <template
            v-for="reward in preparedRewards"
            :key="reward.rewardToken"
          >
            <div class="reward-title">
              This position earns additional market incentives
            </div>

            <div class="reward-info">
              <div class="asset-data">
                <img
                  :src="reward.asset?.icon"
                  alt="asset icon"
                >
                <div class="asset-data__detail">
                  <div class="asset-name">
                    {{ reward.asset?.symbol ?? '-' }} <span>Reward</span>
                  </div>
                  <div class="asset-amount">
                    ${{ formatPrice(reward.rewardPerWeekUSD, 2, 2) }} <span>Weekly</span>
                  </div>
                </div>
              </div>
              <div class="reward-apy">
                {{ truncatePercent(reward.rewardAPY, 2) }}%
              </div>
            </div>
          </template>

          <div class="apy-info">
            <div class="apy-info__item">
              <div class="apy-info__item__title">
                {{ isSupplyFarms ? 'Lending' : 'Borrow' }} APY
              </div>
              <div class="apy-info__item__value">
                {{ truncatePercent(apyData.lendAPY, 2) }}%
              </div>
            </div>
            <div class="apy-info__item">
              <div class="apy-info__item__title">
                Total Combined APY
              </div>
              <div class="apy-info__item__value">
                {{ truncatePercent(apyData.combinedAPY, 2) }}%
              </div>
            </div>
          </div>

          <div class="reward-desc">
            <template v-if="isSupplyFarms">
              Extra rewards earned from market incentive programs. Incentives are added to the lending yield
              and included in the Total Combined APY.
            </template>
            <template v-else>
              Extra rewards earned from market incentive programs. Incentives reduce the effective borrowing cost
              and are included in the Total Combined APY.
            </template>
          </div>
        </template>
      </j-tooltip>
    </template>
    <template v-else>
      {{ apy }}
    </template>
  </j-pill-label>
</template>

<style lang="scss">
.j-pill-label {
  &:has([class*='tooltip']) {
    padding: 0;
  }

  [class*='tooltip'] {
    padding: 2px 10px;
  }

  &.farms-badge {
    &--supply {
      outline: 1px dashed #22d3ee;
    }

    &--borrow {
      outline: 1px dashed #8a8df4;
    }
  }
}

.farms-info-tip {
  .reward-title {
    margin-bottom: $spacing-xl;
    line-height: 16px;
    font-size: 13px;
  }

  .reward-info {
    font-family: $font-JetBrainsMono;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-bottom: $spacing-lg;
    border-bottom: 1px solid $border-secondary;
  }

  .asset-data {
    display: flex;
    align-items: center;
    gap: 8px;

    &__detail {
      display: flex;
      flex-direction: column;
      gap: 4px;

      .asset-name {
        font-size: 14px;
        font-weight: 500;
        text-transform: uppercase;
        color: $navi-50;

        span {
          margin-left: 4px;
        }
      }

      .asset-amount {
        font-size: 12px;
        color: $text-tertiary;
        text-transform: uppercase;

        span {
          margin-left: 4px;
        }
      }
    }

    img {
      width: 24px;
      height: 24px;
      object-fit: contain;
      border-radius: 50%;
    }
  }

  .reward-apy {
    font-size: 14px;
  }

  .apy-info {
    padding-top: $spacing-lg;
    display: flex;
    flex-direction: column;
    gap: 8px;

    &__item {
      display: flex;
      align-items: center;
      justify-content: space-between;
      font-size: 13px;

      &__title {
        color: $navi-50;
      }

      &__value {
        font-family: $font-JetBrainsMono;
      }
    }
  }

  .reward-desc {
    margin-top: $spacing-lg;
    font-size: 12px;
    color: $text-tertiary;
  }
}
</style>
