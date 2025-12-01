<script lang="ts" setup>
const { width } = useWindowSize()
const isSettingsSidebar = ref(false)

provide('isSidebar', isSettingsSidebar)
</script>

<template>
  <j-btn
    rounded
    variant="secondary"
    :size="width > 650 ? 'lg' : 'md'"
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
  cursor: pointer;
  .settings-icon {
    margin-top: 2px;
  }
}

body.body--dark {
  .settings-btn {
    background-color: $neutral-16 !important;
    border-color: $neutral-16 !important;

    .settings-icon {
      path {
        color: #c4c5c7;
      }
    }
  }
}
</style>
