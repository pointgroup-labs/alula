<script lang="ts" setup>

</script>

<template>
  <div class="market-info-card">
    <slot />
  </div>
</template>

<style lang="scss">
.market-info-card {
  display: flex;
  flex-direction: column;
  text-wrap: nowrap;
  justify-content: center;
  align-items: center;
  padding: 16px 24px;
  border: 1px solid #212c42;
  border-radius: 12px;
  font-weight: 500;

  .info-label {
    font-size: 22px;
    line-height: 32px;
  }

  .info-value {
    font-size: 14px;
    line-height: 20px;
    display: flex;
    align-items: center;
    gap: 4px;
  }
}
</style>
