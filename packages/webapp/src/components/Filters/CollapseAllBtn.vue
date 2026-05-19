<script lang="ts" setup>
const { entity = 'accordion-markets' } = defineProps<{ entity?: string }>()
const {
  opened,
  collapseAll,
  showAll,
} = useAccordionMarketsHandler(entity)

const marketsStore = useMarketsStore()
const loading = computed(() => marketsStore.state.loading)

const isCollapsed = ref(false)

function collapseHandler() {
  isCollapsed.value = !isCollapsed.value
  isCollapsed.value ? collapseAll() : showAll()
}

watch(opened, (o) => {
  isCollapsed.value = o.length === 0
}, { immediate: true })
</script>

<template>
  <j-btn
    variant="ghost"
    class="collapse-btn"
    :disabled="loading"
    @click="collapseHandler"
  >
    <i-app-collapse-icon /> {{ isCollapsed ? 'Expand All' : 'Collapse All' }}
  </j-btn>
</template>

<style lang="scss">
.btn.collapse-btn {
  background-color: transparent;
  outline: none !important;
  color: $text-tertiary !important;
  margin-left: auto;
  cursor: pointer;
}
</style>
