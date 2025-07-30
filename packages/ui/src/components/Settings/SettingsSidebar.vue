<script lang="ts" setup>
const { isSidebar } = defineProps<{
  isSidebar: boolean
}>()

const emit = defineEmits(['close'])

function close() {
  emit('close')
}

provide('isSidebar', isSidebar)
</script>

<template>
  <sidebar
    :is-sidebar="isSidebar"
    :title="$t('common.settings')"
    class-name="settings-sidebar"
    @close="close"
  >
    <settings-language />
    <settings-theme />
  </sidebar>
</template>

<style lang="scss">
.settings-sidebar {
  .setting-item {
    padding: $spacing-24 0;

    &:not(:last-child) {
      border-bottom: 1px solid $secondary;
    }
  }
}
</style>
