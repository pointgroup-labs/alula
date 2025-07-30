<script lang="ts" setup>
const isSettingsSidebar = ref(false)
</script>

<template>
  <j-btn
    rounded
    variant="secondary"
    size="lg"
    class="settings-btn"
    @click="isSettingsSidebar = true"
  >
    <i-app-settings-icon class="settings-icon" />
  </j-btn>

  <settings-sidebar
    :is-sidebar="isSettingsSidebar"
    @close="isSettingsSidebar = false"
  />
</template>

<style lang="scss">
.settings-btn {
  cursor: pointer;
  .settings-icon {
    margin-top: 2px;
  }
}
</style>
