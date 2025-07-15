<script setup lang="ts">
const {
  width = '50',
  height = '20',
  variant = 'default',
} = defineProps<{
  width?: string
  height?: string
  variant?: 'default' | 'rounded' | 'square'
  fullWidth?: boolean
}>()
</script>

<template>
  <div
    class="j-skeleton"
    :class="
      [
        `j-skeleton--${variant}`,
        `j-skeleton--${fullWidth ? 'full-width' : 'default'}`,
      ]"
    :style="{ 'width': `${width}px`, 'min-width': `${width}px`, 'height': `${variant !== 'default' ? width : height}px` }"
  />
</template>
