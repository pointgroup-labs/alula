<template>
  <div class="market-overview">
    <market-overview-navigation />

    <div class="overview-wrapper">
      <general-stats />
      <supply-overview />
    </div>

  </div>
</template>

<style lang="scss">
.market-overview {
  display: flex;
  gap: $spacing-16;

  .overview-wrapper {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: $spacing-32;
  }
}
</style>
