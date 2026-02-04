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
  padding: $spacing-16;
  border-radius: $spacing-8;
  background-color: $neutral-2;
  display: flex;
  align-items: flex-start;
  gap: $spacing-8;
  color: $neutral-6;
  font-size: 11px;
  font-style: normal;
  font-weight: 500;
  line-height: 12px;

  .warning-text {
    &--warning {
      color: $danger;
    }
  }

  span {
    font-weight: 700;
  }

  .warning-icon {
    min-width: 16px;
    width: 16px;
    height: 16px;
  }
}

.theme-dark {
  .warning-block {
    background-color: $neutral-18;
    color: $neutral-12;
  }
}
</style>
