<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'

const selectedPool = inject('selectedPool') as Ref<MarketTableItem>

const marketName = computed(() => selectedPool.value?.market)
const pool = computed(() => selectedPool.value?.raw)

const {
  isHaveFarms,

  supplyPreparedRewards,
  borrowPreparedRewards,
} = useFarms({
  marketName,
  pool,
})

const incentiveRewards = computed(() => [...supplyPreparedRewards.value, ...borrowPreparedRewards.value].flat())
</script>

<template>
  <section
    v-if="isHaveFarms"
    id="pool-incentive-overview"
  >
    <div class="pool-card stat-card stat-card--small">
      <div class="stat-card__header">
        <h3 class="title">
          Rewards
        </h3>
      </div>

      <div class="stat-card__body">
        <template
          v-for="reward in incentiveRewards"
          :key="reward.rewardToken"
        >
          <div

            class="incentive-item"
          >
            <img
              :src="reward.asset?.icon"
              alt="reward asset"
            >
            <div class="incentive-item__data">
              <div class="incentive-item__data__title">
                Incentives APY
              </div>
              <div class="incentive-item__data__value">
                {{ truncatePercent(reward.rewardAPY, 2) }}%
              </div>
            </div>
            <div
              class="separator-vert"
            />
            <div class="incentive-item__data">
              <div class="incentive-item__data__title">
                Weekly {{ reward.asset?.symbol }} Rewards
              </div>
              <div class="incentive-item__data__value">
                {{ shortenNumber(reward.rewardPerWeek, 2) }} <span>/ ${{ shortenNumber(reward.rewardPerWeekUSD, 2) }}</span>
              </div>
            </div>
          </div>

        </template>
      </div>
    </div>
  </section>
</template>

<style lang="scss">
section#pool-incentive-overview {
  .stat-card__body {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    row-gap: 22px;

    @media (max-width: $breakpoint-xs) {
      grid-template-columns: 1fr;
    }
  }
  .incentive-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px;
    background-color: $bg-tertiary;
    border-radius: 16px;
    border: 1px solid $border-secondary;

    img {
      width: 32px;
      height: 32px;
      object-fit: contain;
      border-radius: 50%;
    }

    &__data {
      display: flex;
      flex-direction: column;
      gap: 2px;

      &__title {
        font-size: 12px;
        font-style: normal;
        text-transform: uppercase;
        color: $text-tertiary;
      }

      &__value {
        font-family: $font-JetBrainsMono;
        font-size: 14px;
        font-style: normal;
        font-weight: 500;
        color: $text-primary;

        span {
          color: $text-tertiary;
          font-size: 12px;
          font-weight: 500;
          line-height: 14px;
        }
      }
    }

    .separator-vert {
      height: stretch;
      background-color: $navi-400;
    }
  }
}
</style>
