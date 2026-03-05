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
          v-for="value in 3"
          :key="value"
          width="76"
          height="32"
          pill
        />
      </div>
    </div>

    <div class="market-overview">
      <div class="overview-wrapper">
        <section id="stats">
          <div class="stat-cards">
            <j-skeleton
              v-for="value in 5"
              :key="value"
              full-width
              height="72"
              style="border-radius: 12px;"
            />
          </div>
        </section>

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
      />
    </div>
  </div>
</template>
