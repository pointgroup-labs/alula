<script lang="ts" setup>
const tabs = [
  {
    label: 'Positions',
    value: 'positions',
  },
  {
    label: 'Rewards',
    value: 'rewards',
  },
]
const activeTab = ref(tabs[0])
</script>

<template>
  <main class="portfolio-page container">
    <portfolio-info />

    <j-line-tab
      v-model="activeTab"
      :tabs="tabs"
      style="margin-bottom: -12px;"
    >
      <template #tab="{ tab }">
        <span
          class="tab-label"
        >
          {{ tab.label }}
        </span>

      </template>
    </j-line-tab>

    <template v-if="activeTab?.value === 'positions'">
      <div class="portfolio-cards">
        <portfolio-supply-table />
        <portfolio-borrow-table />
      </div>

      <div
        class="portfolio-multiply__cards card"
      >
        <div class="portfolio-card__title">
          My Multiplies
        </div>
        <portfolio-multiply-table only-multiplied />
      </div>
    </template>

    <template v-if="activeTab?.value === 'rewards'">
      <portfolio-rewards-table />
    </template>
  </main>
</template>

<style lang="scss">
@import url(~/assets/styles/components/portfolio.scss);
</style>
