<template>
  <main class="markets-page container">
    <div class="markets-page__title">
      Stellar Lending
    </div>
    <market-info />
    <markets />
  </main>
</template>

<style lang="scss">
.markets-page {
  display: flex;
  flex-direction: column;
  gap: $spacing-24;

  &__title {
    font-size: 20px;
    font-style: normal;
    font-weight: 700;
    line-height: 20px;
  }
}
</style>
