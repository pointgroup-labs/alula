<script lang="ts" setup>
const { width } = useWindowSize()
</script>

<template>
  <div class="market-detail-page container">
    <div class="market-overview">
      <div class="overview-wrapper">
        <j-skeleton
          full-width
          height="300"
          style="border-radius: 12px;"
        />
      </div>
      <j-skeleton
        width="400"
        height="486"
        style="border-radius: 12px;"
        :full-width="width < 1024"
        class="stake-card-skeleton"
      />
    </div>
  </div>
</template>


<style lang="scss">
.market-detail-page {
  .stake-card-skeleton {
    @media (max-width: $breakpoint-xs) {
      min-width: 0 !important;
    }
  }
}
</style>