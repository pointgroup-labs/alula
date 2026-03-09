<script lang="ts" setup>
const { width } = useWindowSize()
const isSettingsSidebar = ref(false)

provide('isSidebar', isSettingsSidebar)
</script>

<template>
  <j-btn
    variant="ghost"
    size="md"
    class="settings-btn"
    aria-haspopup="dialog"
    :aria-expanded="isSettingsSidebar"
    aria-controls="settings-panel"
    aria-label="Open settings"
    type="button"
    @click="isSettingsSidebar = true"
  >
    <client-only>
      <i-app-settings-icon
        v-if="width >= 1024"
        class="settings-icon"
        aria-hidden="true"
      />
      <i-app-menu-icon
        v-else
        class="settings-icon"
        aria-hidden="true"
      />
    </client-only>
  </j-btn>

  <settings-sidebar
    :is-sidebar="isSettingsSidebar"
    @close="isSettingsSidebar = false"
  />
</template>

<style lang="scss">
.settings-btn {
  padding: 8px;
  color: #fff;
  width: 40px;
  height: 40px;
  cursor: pointer;

  .settings-icon {
    width: 18px;
    height: 18px;
  }
}
</style>
