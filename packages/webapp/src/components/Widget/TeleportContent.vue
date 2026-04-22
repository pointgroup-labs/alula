<script lang="ts" setup>
defineProps<{
  to?: string | HTMLElement
}>()
</script>

<template>
  <teleport
    v-if="to"
    :to="to"
  >
    <slot />
  </teleport>
  <template v-else>
    <slot />
  </template>
</template>
