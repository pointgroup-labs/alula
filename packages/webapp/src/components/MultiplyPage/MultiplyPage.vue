<template>
  <main class="page-container container">
    <multiply-info />

    <div class="markets">
      <my-multiply-position-widget />
      <div class="markets-control">
        <div class="market-filters">
          <collateral-assets-filter />
          <debt-assets-filter />
          <clear-filters-btn />
        </div>
        <collapse-all-btn />
        <market-search />
      </div>

      <multiply-table />
    </div>
  </main>
</template>
