<script lang="ts" setup>
defineProps<{
  label: string
  value: string | number
  color: string
}>()
</script>

<template>
  <div
    class="metric-indicator"
    :style="{ '--color': color }"
  >
    <div class="label">
      {{ label }}
    </div>
    <div class="value">
      {{ value }}
    </div>
  </div>
</template>

<style lang="scss">
.metric-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;

  &::before {
    content: '';
    width: 6px;
    height: 6px;
    display: flex;
    border-radius: 50%;
    background-color: var(--color);
  }

  .label {
    color: $muted-foreground;
  }

  .value {
    color: var(--color);
    font-family: $font-JetBrainsMono;
  }
}
</style>
