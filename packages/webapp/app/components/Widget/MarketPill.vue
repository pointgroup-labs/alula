<script lang="ts" setup>
</script>

<template>
  <div
    class="market-pill"
  >
    <slot />
  </div>
</template>

<style lang="scss">
.market-pill {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 4px 12px;
  font-size: $text-xs;
  color: $text-tertiary;
  letter-spacing: 0.05em;
  font-weight: 500;
  text-transform: capitalize;
  background-color: color-mix(in oklab, $secondary 60%, transparent);
  border-radius: $radius-full;

  span {
    color: $text-primary;
  }

  @media (max-width: $breakpoint-xs) {
    padding: 2px 16px;
  }
}
</style>
