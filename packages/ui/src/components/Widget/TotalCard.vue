<script lang="ts" setup>
import defaultIcon from '~/assets/img/icons/strongbox-icon.svg?raw'

const {
  color = '#fff',
  bgColor = '#006CE4',
  icon = defaultIcon,
  loading = false,
} = defineProps<{
  title: string
  body: string
  icon?: string
  color?: string
  bgColor?: string
  loading?: boolean
}>()
</script>

<template>
  <div
    class="total-card"
    :style="{ color, backgroundColor: bgColor }"
  >
    <div class="total-card__info">
      <div class="total-card__title">
        {{ title }}
      </div>
      <j-skeleton
        v-if="loading"
        full-width
        height="28"
      />
      <div
        v-else
        class="total-card__body"
      >
        {{ body }}
      </div>
    </div>

    <div
      class="total-card__icon"
      :style="{ color: bgColor }"
      v-html="icon"
    />
  </div>
</template>

<style lang="scss">
.total-card {
  position: relative;
  width: 340px;
  height: 168px;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  color: #fff;
  background-color: $primary;
  padding: $spacing-32;
  border-radius: $spacing-16;
  overflow: hidden;

  &::before {
    content: '';
    position: absolute;
    bottom: -8px;
    height: 76px;
    width: 100%;
    left: -8px;
    background: url(~/assets/img/card/collateral-bg.svg) no-repeat;
    background-size: contain;
    background-position: bottom;
  }

  &__info {
    display: flex;
    flex-direction: column;
    gap: $spacing-8;
    font-style: normal;
  }

  &__title {
    font-size: 20px;
    font-weight: 500;
    line-height: 20px;
  }

  &__body {
    font-size: 32px;
    font-style: normal;
    font-weight: 700;
    line-height: 28px;
  }

  &__icon {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: #fff;
  }
}
</style>
