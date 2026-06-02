<script lang="ts" setup>
const {
  color = 'rgba(255, 255, 255, 0.3)',
  borderWidth = '2px',
  width = '30px',
} = defineProps<{
  color?: string
  borderWidth?: string
  fixed?: boolean
  blur?: boolean
  width?: string
}>()
</script>

<template>
  <div
    class="loading-spinner"
    :class="{
      'loading-spinner--fixed': fixed,
      'loading-spinner--blur': blur,
    }
    "
  >
    <b-spinner :style="{ color, 'border-width': borderWidth, width, 'height': width }" />
    <slot />
  </div>
</template>
