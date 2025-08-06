<script lang="ts" setup>
const tabs: Record<string, string>[] = inject('navTabs', [])
</script>

<template>
  <div
    class="setting-item navigation"
  >
    <nuxt-link
      v-for="(tab, index) in tabs"
      :key="index"
      class="navigation-item"
      :to="String(tab?.route)"
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
  gap: $spacing-4;

  .navigation-item {
    padding: $spacing-16;
    border-radius: $spacing-8;
    background-color: $neutral-2;
    color: $dark;
    font-size: 16px;
    font-style: normal;
    font-weight: 500;
    line-height: 20px;
    text-decoration: none;
    display: flex;
    align-items: center;
    gap: $spacing-16;

    i {
      width: 32px;
      height: 32px;
      display: flex;
      align-items: center;
      justify-content: center;
      border-radius: $spacing-8;
      background-color: $neutral-3;

      svg {
        width: 20px;
        height: 20px;
      }
    }
  }
}

body.body--dark {
  .setting-item.navigation {
    .navigation-item {
      background-color: $neutral-18;
      color: $neutral-12;

      i {
        background-color: $neutral-16;
        color: #fff;
      }
    }
  }
}
</style>
