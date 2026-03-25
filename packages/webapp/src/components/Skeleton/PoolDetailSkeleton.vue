<script lang="ts" setup>
const { width } = useWindowSize()
</script>

<template>
  <div class="market-detail-page container">
    <div class="market-details-top">
      <j-skeleton
        variant="rounded"
        width="38"
        height="38"
      />

      <j-skeleton
        pill
        width="100"
        height="22"
      />
      <div
        class="pool-metrics"
        style="height: 45px;"
      >
        <j-skeleton
          v-for="value in 2"
          :key="value"
          width="76"
          height="48"
          pill
        />
      </div>
    </div>

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
      />
    </div>
  </div>
</template>
