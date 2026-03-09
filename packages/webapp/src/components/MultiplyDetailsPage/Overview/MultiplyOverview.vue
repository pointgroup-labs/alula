<template>
  <div class="multiply-overview">
    <div class="multiply-overview__wrapper">
      <multiply-stats />
      <multiply-history-chart />
      <multiply-vault />
    </div>
    <multiply-form />
  </div>
</template>

<style lang="scss">
.multiply-overview {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  padding-bottom: 120px;

  &__wrapper {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 64px;
  }
}
</style>
