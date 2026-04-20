<template>
  <main class="portfolio-page container">
    <div class="portfolio-cards card">
      <portfolio-supply-table />
      <portfolio-borrow-table />
    </div>

    <div class="portfolio-multiply__cards card">
      <div class="portfolio-card__title">
        My Multiplied Positions
      </div>
      <portfolio-multiply-table only-multiplied />
    </div>
  </main>
</template>

<style lang="scss">
.portfolio-page {
  .portfolio-cards {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    background-color: transparent;
    border: none;

    @media (max-width: $breakpoint-sm) {
      display: flex;
      flex-direction: column;
      gap: 16px;
    }
  }

  .portfolio-multiply__cards {
    width: calc(100% - 48px);
    margin: 0 auto;
    padding: 0;
  }
}
</style>
