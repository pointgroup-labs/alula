<script lang="ts" setup>
const { tabs = [] } = defineProps<{
  tabs?: { label: string, value: string }[]
}>()

const activeTab = defineModel<{ label: string, value: string }>()

watchEffect(() => {
  if (tabs?.length) {
    activeTab.value = tabs[0]
  }
})
</script>

<template>
  <div class="market-overview-tabs">
    <div
      v-for="tab in tabs"
      :key="tab.label"
      class="overview-tab"
      :class="{ 'overview-tab--active': activeTab?.value === tab?.value }"
      @click="activeTab = tab"
    >
      {{ tab.label }}
    </div>
  </div>
</template>

<style lang="scss">
.market-overview-tabs {
  display: flex;
  align-items: flex-end;
  gap: $spacing-12;
  font-weight: 500;

  .overview-tab {
    padding: 0 $spacing-6 $spacing-12;
    border-bottom: 2px solid transparent;
    transition: 0.1s ease;
    cursor: pointer;

    &--active {
      border-color: $primary;
    }
  }
}
</style>
