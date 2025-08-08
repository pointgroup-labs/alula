<script lang="ts" setup>
const props = defineProps<{
  to: string
  isTeleport?: boolean
}>()

const targetEl = shallowRef<HTMLElement | null>(null)
const ready = ref(false)
let observer: MutationObserver | null = null

const resolveTarget = () => {
  const el = document.querySelector(props.to) as HTMLElement | null
  if (el && el !== targetEl.value) {
    targetEl.value = el
    requestAnimationFrame(() => { ready.value = true })
  }
}

onMounted(() => {
  nextTick().then(() => {
    resolveTarget()
    observer = new MutationObserver(resolveTarget)
    observer.observe(document.body, { childList: true, subtree: true })
  })
})

onBeforeUnmount(() => observer?.disconnect())

watch(() => props.to, () => {
  ready.value = false
  targetEl.value = null
  resolveTarget()
})
</script>

<template>
  <ClientOnly>
    <teleport
      v-if="isTeleport && ready && targetEl"
      :key="`${props.to}::${!!targetEl}`"
      :to="targetEl"
    >
      <slot />
    </teleport>
    <slot v-else />
  </ClientOnly>
</template>
