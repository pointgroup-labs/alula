<script lang="ts" setup>
</script>

<template>
  <main class="portfolio-page container">
    <div class="portfolio-cards">
      <portfolio-supply-table />
      <portfolio-borrow-table />
    </div>

    <features-toggle feature="multiply">
      <div
        class="portfolio-multiply__cards card"
      >
        <div class="portfolio-card__title">
          My Multiplied Positions
        </div>
        <portfolio-multiply-table only-multiplied />
      </div>
    </features-toggle>
  </main>
</template>

<style lang="scss">
.portfolio-page {
  .portfolio-cards {
    display: flex;
    flex-direction: row;
    gap: 16px;
    background-color: transparent;
    border: none;

    @media (max-width: $breakpoint-sm) {
      display: flex;
      flex-direction: column;
      gap: 16px;
    }

    .table-wrapper {
      height: 100%;
      min-height: 97px;
      max-height: 500px;
    }
  }

  .portfolio-multiply__cards {
    width: 100%;
    margin: 0 auto;
    padding: 0;
  }
}
</style>
