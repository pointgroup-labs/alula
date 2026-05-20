<script lang="ts" setup>
const {
  variant = 'cyan',
  size = 'lg',
} = defineProps<{
  color?: string
  bgColor?: string
  label?: string
  variant?: 'cyan' | 'indigo' | 'success' | 'danger'
  size?: 'lg' | 'md' | 'sm'
}>()
</script>

<template>
  <div
    class="j-pill-label"
    :class="[
      `j-pill-label--${variant}`,
      `j-pill-label--${size}`,
    ]"
    :style="{ color, backgroundColor: bgColor }"
  >
    <template v-if="label">
      {{ label }}
    </template>
    <slot v-else />
  </div>
</template>
