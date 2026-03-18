<script lang="ts" setup>
const emits = defineEmits(['close'])

const tabs: Record<string, string>[] = inject('navTabs', [])
const route = useRoute()

function isActiveRoute(tab: Record<string, string>) {
  return tab.route === route.path
}
</script>

<template>
  <div
    class="setting-item navigation"
  >
    <nuxt-link
      v-for="(tab, index) in tabs"
      :key="index"
      class="navigation-item"
      :class="{ 'navigation-item--active': isActiveRoute(tab) }"
      :to="String(tab?.route)"
      @click="emits('close')"
    >
      <i v-html="tab.icon" />
      {{ tab?.shortLabel || tab.label }}
    </nuxt-link>
  </div>
</template>

<style lang="scss">
.setting-item.navigation {
  display: flex;
  flex-direction: column;
  gap: 4px;

  .navigation-item {
    padding: $spacing-xl;
    border-radius: $radius-md;
    color: $text-primary;
    font-size: 16px;
    font-style: normal;
    font-weight: 500;
    line-height: 20px;
    text-decoration: none;
    display: flex;
    align-items: center;
    gap: 16px;

    &--active {
      background-color: $surface-neutral-08;
    }

    i {
      width: 32px;
      height: 32px;
      display: flex;
      align-items: center;
      justify-content: center;
      border-radius: $radius-md;
      background-color: $navi-400;
      color: $navi-50;

      svg {
        width: 20px;
        height: 20px;
      }
    }
  }
}
</style>
