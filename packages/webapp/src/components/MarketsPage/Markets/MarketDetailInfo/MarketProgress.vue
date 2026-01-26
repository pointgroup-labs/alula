<script lang="ts" setup>
import { shortenNumber } from '~/utils'

const {
  progress,
  color = '#006CE4',
  isProgress = false,
  limit = 0,
  symbol,
}
  = defineProps<{
    progress: number | string
    isProgress?: boolean
    detailsColor?: string
    color?: string
    cap?: number
    limit?: number
    symbol?: string
  }>()

const limitData = computed(() => limit > 0 ? shortenNumber(limit) : '-')
const poolLimitText = computed(() => {
  return limit ? `Pool limit is ${formatPrice(limit)} ${symbol}` : 'Pool limit not set'
})
</script>

<template>
  <div class="market-progress__wrapper">
    <div class="market-progress">
      <j-circular-progress
        v-if="isProgress"
        :progress="Number(progress)"
        :width="70"
        :stroke-width="25"
        :stroke-bg="isDark ? '#262729' : '#EAECF0'"
        :stroke-color="color"
        :background="isDark ? '#111' : '#fff'"
        :color="isDark ? '#fff' : '#111'"
      />

      <slot />
    </div>

    <div class="separator" />

    <div
      class="market-progress__details"
      :style="{ '--color': detailsColor }"
    >
      <div class="market-cap">
        Cap :   {{ shortenNumber(cap || 0) }}
      </div>
      <div class="market-limit">
        Limit :   {{ limitData }}
        <info-tooltip
          :text="poolLimitText"
          :size="12"
        />
      </div>
    </div>
  </div>
</template>

<style lang="scss">
.market-progress__wrapper {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: $spacing-8;
}
.market-progress {
  height: 70px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: $spacing-24;

  .j-circular-progress {
    font-size: 10px;
    font-style: normal;
    font-weight: 500;
    line-height: 12px;
  }

  &__info {
    display: flex;
    flex-direction: column;
    gap: $spacing-8;

    &__title {
      font-size: 14px;
      font-style: normal;
      font-weight: 700;
      line-height: 16px;
    }

    &__data {
      display: flex;
      flex-direction: column;
      align-items: flex-end;
      font-size: 13px;
      font-style: normal;
      font-weight: 500;
      line-height: 14px;

      span {
        color: $neutral-9;
        font-size: 10px;
        font-weight: 500;
        line-height: 14px;
      }
    }
  }

  &__details {
    font-size: 13px;
    font-style: normal;
    font-weight: 500;
    line-height: 14px;
    display: flex;
    gap: $spacing-8;
    justify-content: space-between;

    .market-cap {
      color: var(--color, $dark);
      display: flex;
      align-items: center;
    }
    .market-limit {
      display: flex;
      align-items: center;
      gap: 4px;
    }
  }
}
</style>
