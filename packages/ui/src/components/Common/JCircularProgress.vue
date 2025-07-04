<script lang="ts" setup>
const {
  progress = 0,
  strokeWidth = 20,
  strokeColor = '#ffd600',
  strokeBg = '#ddd',
  width = 70,
  background = '#fff',
  color = '#111',
} = defineProps<{
  strokeWidth?: number
  strokeColor?: string
  strokeBg?: string
  width?: number | string
  background?: string
  color?: string
  progress?: number
}>()

const current = ref(0)
const firstLoad = ref(true)

watch(() => progress, () => {
  if (!firstLoad.value) {
    current.value = progress
    return
  }
  if (progress && firstLoad.value) {
    setTimeout(() => {
      current.value = progress
      firstLoad.value = false
    }, 500)
  }
}, { immediate: true })
</script>

<template>
  <div
    class="j-circular-progress"
    :style="{
      width: `${width}px`,
      height: `${width}px` }"
  >
    <div
      class="progress-data"
      :style="{ '--stroke-width': `${strokeWidth}px`,
                'background': background,
                'color': color }"
    >
      {{ progress }}%
    </div>
    <svg
      width="250" height="250" viewBox="0 0 250 250" class="circular-progress"
      :style="{
        '--current': current,
        '--stroke-width': `${strokeWidth}px`,
        '--stroke-color': strokeColor,
        '--stroke-background': strokeBg,
        'width': `${width}px`,
        'height': `${width}px`,
      }"
    >
      <circle class="bg" />
      <circle class="fg" />
    </svg>
  </div>
</template>
