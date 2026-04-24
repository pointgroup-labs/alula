<script lang="ts" setup>
const tabs = [
  {
    label: 'Pool Info',
    value: 'info',
  },
  {
    label: 'Pool Details',
    value: 'details',
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

    <pool-info-overview v-if="activeTab?.value === 'info'" />
    <pool-detail-overview v-if="activeTab?.value === 'details'" />
    <pool-audits-overview v-if="activeTab?.value === 'audits'" />
  </section>
</template>

<style lang="scss">
section#pool-details-bottom {
  .j-line-tabs {
    position: relative;
    margin-bottom: -8px;
  }

  .stat-card__body {
    overflow-x: auto;
  }
}
</style>
