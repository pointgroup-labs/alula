<script lang="ts" setup>
const tabs = [
  {
    label: 'Pool Info',
    value: 'info',
  },
  {
    label: 'Audits',
    value: 'audits',
  },
]

const activeTab = ref(tabs[0])
</script>

<template>
  <section
    id="pool-details-bottom"
  >
    <j-line-tab
      v-model="activeTab"
      :tabs="tabs"
    />

    <multiply-pool-info-overview v-if="activeTab?.value === 'info'" />
    <multiply-pool-audits-overview v-if="activeTab?.value === 'audits'" />
  </section>
</template>

<style lang="scss">
section#pool-details-bottom {
  display: flex;
  flex-direction: column;
  gap: 32px;

  .j-line-tabs {
    position: relative;
    margin-bottom: -8px;
  }

  .stat-card__body {
    overflow-x: auto;
  }
}
</style>
