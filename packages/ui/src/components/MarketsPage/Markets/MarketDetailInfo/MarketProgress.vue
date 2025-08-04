<script lang="ts" setup>
import { shortenNumber } from '~/utils'

const {
  progress,
  color = '#006CE4',
  isProgress = false,
  limit = 0,
}
= defineProps<{
  color?: string
  cap?: number
  limit?: number
  progress: number | string
  isProgress?: boolean
  detailsColor?: string
}>()

const limitData = computed(() => limit > 0 ? shortenNumber(limit) : '∞')
</script>

<template>
  <div class="market-progress__wrapper">
    <div class="market-progress">
      <j-circular-progress
        v-if="isProgress"
        :progress="Number(progress)"
        :width="60"
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
      </div>
    </div>
  </div>
</template>

<style lang="scss">
.market-progress__wrapper {
  display: flex;
  flex-direction: column;
  gap: $spacing-8;
}
.market-progress {
  height: 60px;
  display: flex;
  align-items: center;
  gap: $spacing-12;

  .j-circular-progress {
    font-size: 10px;
    font-style: normal;
    font-weight: 500;
    line-height: 12px;
  }

  &:not(.j-circular-progress) {
    & {
      justify-content: flex-end;
    }
    .market-progress__info {
      align-items: flex-end;
    }
  }

  &__info {
    display: flex;
    flex-direction: column;
    gap: $spacing-8;

    &__title {
      font-size: 12px;
      font-style: normal;
      font-weight: 700;
      line-height: 16px;
    }

    &__data {
      display: flex;
      flex-direction: column;
      align-items: flex-end;
      font-size: 11px;
      font-style: normal;
      font-weight: 500;
      line-height: 12px;

      span {
        color: $neutral-9;
        font-size: 10px;
        font-weight: 500;
        line-height: 12px;
      }
    }
  }

  &__details {
    font-size: 11px;
    font-style: normal;
    font-weight: 500;
    line-height: 12px;
    display: flex;
    gap: $spacing-8;
    justify-content: space-between;

    .market-cap {
      color: var(--color, $dark);
    }
  }
}
</style>
