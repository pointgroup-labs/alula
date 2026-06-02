<script lang="ts" setup>
const {
  size = 18,
  stroke = 2.2,
} = defineProps<{
  color?: string
  bgColor?: string
  size?: number
  stroke?: number
}>()

const RADIUS = (size - stroke) / 2
const CIRCUMFERENCE = 2 * Math.PI * RADIUS

const poolCountdown = inject<Ref<number>>('poolCountdown')

const dashoffset = computed(() => {
  const fraction = (poolCountdown?.value ?? 0) / 30
  return CIRCUMFERENCE * fraction
})

const progressCircle = useTemplateRef<SVGCircleElement>('progressCircle')

watch(poolCountdown ?? ref(0), (val, prev) => {
  if (val > prev) {
    const el = progressCircle.value
    if (!el) {
      return
    }
    el.style.transition = 'none'
    el.getBoundingClientRect()
    requestAnimationFrame(() => {
      el.style.transition = ''
    })
  }
})
</script>

<template>
  <div
    class="reload-coutdown__progress"
    :style="{ '--color': color, '--track-color': bgColor }"
  >
    <svg
      :width="size"
      :height="size"
      class="reload-countdown"
    >
      <circle
        :cx="size / 2"
        :cy="size / 2"
        :r="RADIUS"
        fill="none"
        class="reload-countdown__track"
        :stroke-width="stroke"
      />
      <circle
        ref="progressCircle"
        :cx="size / 2"
        :cy="size / 2"
        :r="RADIUS"
        fill="none"
        class="reload-countdown__progress"
        :stroke-width="stroke"
        :stroke-dasharray="CIRCUMFERENCE"
        :stroke-dashoffset="dashoffset"
        stroke-linecap="round"
        :transform="`rotate(-90 ${size / 2} ${size / 2})`"
      />
    </svg>
  </div>
</template>

<style scoped>
.reload-coutdown__progress {
  display: flex;
  align-items: center;
  justify-content: center;
}
.reload-countdown__track {
  stroke: color-mix(in srgb, var(--track-color, currentColor) 20%, transparent);
}

.reload-countdown__progress {
  stroke: var(--color, currentColor);
  transition: stroke-dashoffset 0.9s linear;
}
</style>
