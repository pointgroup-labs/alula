<script lang="ts" setup>
import { useMcpSettings } from '../composables/settings'

const { settings } = useMcpSettings()
</script>

<template>
  <j-popover
    class-name="mcp-settings"
    position="bottom"
    body-class="mcp-settings-menu"
    :teleport="false"
  >
    <div class="mcp-settings-list">
      <div class="mcp-settings-item">
        Glass: <j-toggle
          v-model="settings.isGlass"
          size="sm"
        />
      </div>
    </div>
    <template #target>
      <i-app-settings-icon class="settings-icon" />
    </template>
  </j-popover>
</template>

<style lang="scss">
.mcp-settings {
  margin-left: auto;
  display: flex;
  align-items: center;
  justify-content: center;

  .settings-icon {
    width: 16px;
    height: 16px;
    cursor: pointer;
  }
}

.mcp-settings-menu {
  &.popover-body {
    padding: 12px;
  }
  .mcp-settings-list {
    display: flex;
    flex-direction: column;

    .mcp-settings-item {
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 12px;
      color: $navi-100;
    }
  }
}
</style>
