<script lang="ts" setup>
import borrowingIcon from '~/assets/img/icons/percentage-square-icon.svg?raw'
</script>

<template>
  <div class="market-info">
    <total-card title="Total Collateral" body="$65,62" />
    <total-card title="Total Borrowing" body="$28,44" color="#111" bg-color="#FFD101" :icon="borrowingIcon" />

    <div class="total-card market-size">
      <div class="total-card__info">
        <div class="total-card__title">
          Total market size
        </div>
        <div class="total-card__body">
          $23,123B
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="scss">
.market-info {
  display: flex;
  align-items: center;
  gap: $spacing-24;

  .market-size {
    color: $dark;
    background-color: transparent;
    justify-content: flex-end;
    margin-left: auto;

    &::before {
      display: none;
    }
  }
}
</style>
