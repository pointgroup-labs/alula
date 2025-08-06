<script lang="ts" setup>
const { isSidebar } = defineProps<{
  isSidebar: boolean
}>()

const emit = defineEmits(['close'])

function close() {
  emit('close')
}
</script>

<template>
  <sidebar
    :is-sidebar="isSidebar"
    :title="$t('common.settings')"
    class-name="settings-sidebar"
    @close="close"
  >
    <settings-connect />
    <settings-language />
    <settings-theme />
  </sidebar>
</template>

<style lang="scss">
.settings-sidebar {
  .sidebar-body {
    display: flex;
    flex-direction: column;
    gap: $spacing-24;
    padding-top: $spacing-24;
  }

  .setting-item__title {
    font-size: 16px;
    font-style: normal;
    font-weight: 700;
    line-height: 20px;
  }
}

body.body--dark {
}
</style>
