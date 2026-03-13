<script lang="ts" setup>
const {
  progress,
  color = '#006CE4',
  isProgress = false,
}
  = defineProps<{
    progress: number | string
    isProgress?: boolean
    color?: string
    isInfinity?: boolean
  }>()

const slot = useSlots()
</script>

<template>
  <div class="market-progress__wrapper">
    <div class="market-progress">
      <j-circular-progress
        v-if="isProgress"
        :progress="Number(progress)"
        :width="48"
        :stroke-width="18"
        :stroke-color="color"
        stroke-bg="#262729"
        background="transparent"
        :color="color"
      >
        <template
          v-if="slot?.progress"
          #progress
        >
          <slot name="progress" />
        </template>
      </j-circular-progress>

      <slot />
    </div>
  </div>
</template>

<style lang="scss">
.market-progress__wrapper {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}
.market-progress {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;

  .j-circular-progress {
    font-size: 8px;
    font-style: normal;
    font-weight: 500;
    line-height: 12px;
  }

  &__info {
    display: flex;
    flex-direction: column;
    gap: 2px;

    &__title {
      font-size: $text-xs;
      font-style: normal;
      text-transform: uppercase;
      color: $text-tertiary;
    }

    &__data {
      font-family: $font-JetBrainsMono;
      font-size: 14px;
      font-style: normal;
      font-weight: 500;
      line-height: normal;

      span {
        color: $text-tertiary;
        font-size: $text-xs;
        font-weight: 500;
        line-height: 14px;
      }
    }
  }
}
</style>
