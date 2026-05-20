<script lang="ts" setup>
/**
 * Usage:
 *   <sidebar-panel title="sidebar title">
 *     <template #trigger>
 *       <!-- what is visible in the sidebar and clicked to open -->
 *     </template>
 *
 *     <!-- content that will open inside the sidebar when clicked -->
 *   </sidebar-panel>
 */

type PanelView = {
  title: string
  render: () => any
}

const props = defineProps<{
  title: string
}>()

const slots = useSlots()

const push = inject<(view: PanelView) => void>('sidebarPush')

function open() {
  push?.({
    title: props.title,
    render: () => slots.default?.(),
  })
}
</script>

<template>
  <div
    class="sidebar-panel"
    @click="open"
  >
    <slot name="trigger" />
  </div>
</template>

<style lang="scss">
.sidebar-panel {
  display: contents;
}
</style>
