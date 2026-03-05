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
  </section>
</template>

<style lang="scss">
section#pool-details-bottom {
  .j-line-tabs {
    position: relative;
    margin-bottom: -8px;

    &::before {
      content: '';
      position: absolute;
      bottom: 0;
      left: 0;
      right: 0;
      height: 1px;
      background-color: $border-color;
    }
  }
}
</style>
