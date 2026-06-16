<script lang="ts" setup>
import type { PoolData } from '@alula/market-sdk'
import { toRef } from 'vue'

const props = withDefaults(defineProps<{
  apy?: string | number
  variant?: 'cyan' | 'indigo' | 'success' | 'danger'
  size?: 'lg' | 'md' | 'sm'
  farmType: 'supply' | 'borrow'
  poolData?: PoolData
  marketName?: string
}>(), {
  variant: 'cyan',
  size: 'sm',
})

const slots = useSlots()

const {
  supplyApyData,
  borrowApyData,
  supplyActualRewards,
  borrowActualRewards,

  supplyPreparedRewards,
  borrowPreparedRewards,
} = useFarms({
  marketName: toRef(props, 'marketName'),
  pool: toRef(props, 'poolData'),
})

const isSupplyFarms = computed(() => props.farmType === 'supply')

const actualRewards = computed(() => isSupplyFarms.value ? supplyActualRewards.value : borrowActualRewards.value)
const preparedRewards = computed(() => isSupplyFarms.value ? supplyPreparedRewards.value : borrowPreparedRewards.value)
const apyData = computed(() => isSupplyFarms.value ? supplyApyData.value : borrowApyData.value)

const farmsClasses = computed(() => {
  if (actualRewards.value.length === 0) {
    return []
  }

  return [`farms-badge--${props.farmType}`]
})
</script>

<template>
  <j-pill-label
    :variant="variant"
    :size="size"
    :class="farmsClasses"
    class="farms-badge"
  >
    <template v-if="actualRewards.length > 0">
      <j-tooltip content-class="farms-info-tip">
        {{ truncatePercent(Math.max(apyData.combinedAPY, 0), 2) }}% <i-app-lighting-icon />

        <template #content>
          <div class="reward-title">
            This position earns additional market incentives
          </div>

          <template
            v-for="reward in preparedRewards"
            :key="reward.rewardToken"
          >

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
                    ${{ shortenNumber(reward.rewardPerWeekUSD, 2, 2) }} <span>Weekly</span>
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
              <div
                class="apy-info__item__value"
              >
                {{ truncatePercent(Math.max(apyData.combinedAPY, 0), 2) }}%
              </div>
            </div>

            <div
              v-if="apyData.combinedAPY < 0 && !isSupplyFarms"
              class="apy-info__item"
            >
              <div class="apy-info__item__title">
                Extra Rewards APY
              </div>
              <div
                class="apy-info__item__value"
              >
                +{{ truncatePercent(Math.abs(apyData.combinedAPY), 2) }}%
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
    <template v-else-if="slots.default">
      <slot />
    </template>
    <template v-else>
      {{ apy }}
    </template>
  </j-pill-label>
</template>

<style lang="scss">
.farms-badge {
  &:has([class*='tooltip']) {
    padding: 0;
  }

  [class*='tooltip'] {
    padding: 2px 10px;
  }

  &--supply {
    outline: 1px dashed #22d3ee;
  }

  &--borrow {
    outline: 1px dashed #8a8df4;
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
    margin-top: $spacing-xl;
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
