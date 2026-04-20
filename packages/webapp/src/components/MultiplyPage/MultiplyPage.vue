<template>
  <main class="page-container container">
    <multiply-info />

    <div class="markets">
      <my-multiply-position-widget />
      <div class="markets-control">
        <div class="market-filters">
          <collateral-assets-filter scope="multiply" />
          <debt-assets-filter scope="multiply" />
          <clear-filters-btn scope="multiply" />
        </div>
        <collapse-all-btn />
        <market-search scope="searchMultiply" />
      </div>

      <multiply-table />
    </div>
  </main>
</template>
