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
    <i-app-warning-icon class="warning-icon" />
    <div
      class="warning-text"
      :class="[`warning-text--${isWarning ? 'warning' : 'default'}`]"
    >
      <span>{{ title }}</span>
      {{ text }}
    </div>
  </div>
</template>

<style lang="scss">
.warning-block {
  padding: $spacing-xl;
  display: flex;
  align-items: flex-start;
  gap: 8px;
  color: $text-primary;
  font-size: 12px;
  font-style: normal;
  font-weight: 400;
  line-height: 16px;
  border-radius: $radius-md;
  border: 1px solid $yellow-500;
  background: #3a2312;

  .warning-text {
    display: flex;
    flex-direction: column;
    gap: 8px;
    overflow-wrap: anywhere;
    word-break: break-word;

    // &--warning {
    // }
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
</style>
