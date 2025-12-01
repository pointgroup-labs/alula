<template>
  <div class="market-info__bottom">
    <market-interest />
    <market-interest-chart />
    <div class="separator-vert hide-xs" />
    <logo-with-text class="hide-xs" />
  </div>
</template>

<style lang="scss">
.market-info-dialog {
  .market-info__bottom {
    display: flex;
    gap: $spacing-16;

    @media (max-width: $breakpoint-xs) {
      flex-direction: column;
      gap: $spacing-8;
    }

    .logo-with-text {
      flex: 1;
      justify-content: center;
    }
  }
}
</style>
