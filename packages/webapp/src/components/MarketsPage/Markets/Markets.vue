<template>
  <div class="markets card">
    <market-filters />
    <market-table />
  </div>
</template>

<style lang="scss">
.markets {
  display: flex;
  flex-direction: column;
  gap: $spacing-16;

  @media (max-width: $breakpoint-sm) {
    gap: 10px;
  }

  .markets-tabs {
    width: fit-content;

    .btn {
      min-width: 200px;

      &-content {
        gap: $spacing-4;
      }
    }

    svg {
      width: 24px;
      height: 24px;
    }
  }
}
</style>
