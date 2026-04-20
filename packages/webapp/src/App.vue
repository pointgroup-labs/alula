<script lang="ts" setup>
const { start, poolCountdown, obligationCountdown } = useSmartReloader()
provide('poolCountdown', poolCountdown)
provide('obligationCountdown', obligationCountdown)

onMounted(() => {
  if (import.meta.client) {
    nextTick(() => {
      const body = document.querySelector('body') as HTMLElement
      if (body) {
        body.style.transition = 'opacity 0.3s ease-in-out'
        body.style.opacity = '1'
      }
    })
    start()
  }
})
</script>

<template>
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
  <b-toast-orchestrator />
</template>
