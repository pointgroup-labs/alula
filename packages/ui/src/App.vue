<script lang="ts" setup>
onMounted(() => {
  if (import.meta.client) {
    nextTick(() => {
      const body = document.querySelector('body') as HTMLElement
      if (body) {
        body.style.transition = 'opacity 0.3s ease-in-out'
        body.style.opacity = '1'
      }
    })
    const { start } = useSmartReloader()
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
