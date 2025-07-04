<script lang="ts" setup>
</script>

<template>
  <div class="market-details">
    <div class="market-details__title">
      Borrow Info
    </div>

    <div class="market-stats">
      <div class="market-stats__apy">
        <div class="stats-apy">
          Borrow APY
          <span>7.28%</span>
        </div>
        <div class="stats-params">
          <div class="stats-params__item">
            Borrow Cap:
            <span>45%</span>
          </div>
          <div class="stats-params__item">
            Reserve:
            <span>24%</span>
          </div>
        </div>
      </div>

      <div class="separator-vert" />

      <market-progress :progress="92.31" color="#FFD101">
        <div class="market-progress__info">
          <div class="market-progress__info__title">
            Total Borrowed
          </div>
          <div class="market-progress__info__data">
            327.92K / 800K

            <span>$1327.92K / $1800K</span>
          </div>
        </div>
      </market-progress>

      <div class="separator-vert" />

      <a href="#" target="_blank" class="market-penalty justify-content-center">
        View contract

        <i-app-export-icon color="#111" />
      </a>
    </div>

    <div class="separator" />

    <market-history-chart-borrow />
  </div>
</template>

<style lang="scss" scoped>
.market-penalty {
  gap: 10px;
  text-decoration: none;
}
</style>
