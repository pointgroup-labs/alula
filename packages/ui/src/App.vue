<script lang="ts" setup>
onMounted(() => {
  nextTick(() => {
    const body = document.querySelector('body') as HTMLElement
    if (body) {
      body.style.transition = 'opacity 0.3s ease-in-out'
      body.style.opacity = '1'
    }
  })
})
</script>

<template>
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
  <b-toast-orchestrator />
</template>
