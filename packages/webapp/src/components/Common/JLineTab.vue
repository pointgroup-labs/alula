<script lang="ts" setup>
const { tabs = [] } = defineProps<{
  tabs?: { label: string, value: string }[]
  activeColor?: string
}>()

const activeTab = defineModel<{ label: string, value: string }>()

watchEffect(() => {
  if (tabs?.length) {
    activeTab.value = tabs[0]
  }
})
</script>

<template>
  <div class="j-line-tabs">
    <div
      v-for="tab in tabs"
      :key="tab.label"
      class="overview-tab"
      :class="{ 'overview-tab--active': activeTab?.value === tab?.value }"
      :style="{ '--active-tab-color': activeColor }"
      @click="activeTab = tab"
    >
      {{ tab.label }}
    </div>
  </div>
</template>

<style lang="scss">
.j-line-tabs {
  position: relative;
  display: flex;
  align-items: flex-end;
  gap: 12px;
  font-size: 14px;
  font-weight: 500;

  @media (max-width: $breakpoint-sm) {
    justify-content: center;
  }

  &:before {
    content: '';
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 1px;
    background-color: color-mix(in oklab, #1e293b 40%, transparent);
  }

  .overview-tab {
    padding: 0 $spacing-sm $spacing-md;
    border-bottom: 2px solid transparent;
    transition: 0.1s ease;
    cursor: pointer;

    &--active {
      border-color: var(--active-tab-color, $text-tertiary);
    }
  }
}
</style>
