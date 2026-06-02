<template>
  <div class="markets">
    <my-positions-widget />
    <div class="markets-control">
      <div class="market-filters">
        <collateral-assets-filter />
        <debt-assets-filter />
        <clear-filters-btn />
      </div>
      <collapse-all-btn />
      <market-search />
    </div>
    <markets-list />
  </div>
</template>
