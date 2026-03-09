<script lang="ts" setup>
const {
  title = 'Attention',
  isWarning = false,
} = defineProps<{
  text: string
  title?: string
  isWarning?: boolean
}>()
</script>

<template>
  <div class="warning-block">
    <i-app-warning-color class="warning-icon" />
    <div :class="[`warning-text--${isWarning ? 'warning' : 'default'}`]">
      <span>{{ title }}:</span> {{ text }}
    </div>
  </div>
</template>

<style lang="scss">
.warning-block {
  padding: 10px $spacing-xl;
  display: flex;
  align-items: center;
  gap: 8px;
  color: $text-primary;
  font-size: 12px;
  font-style: normal;
  font-weight: 400;
  line-height: 16px;
  border-radius: $radius-2xl;
  border: 1px solid $warning;
  background: rgba(228, 156, 11, 0.15);

  .warning-text {
    &--warning {
      color: $danger;
    }
  }

  span {
    font-weight: 700;
  }

  .warning-icon {
    min-width: 24px;
    width: 24px;
    height: 24px;
  }
}
</style>
