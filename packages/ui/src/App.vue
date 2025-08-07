<script lang="ts" setup>
function setFavicon(darkMode: boolean) {
  const favicon = document.createElement('link')
  favicon.rel = 'icon'
  favicon.href = darkMode ? '/favicon-dark.svg' : '/favicon.svg'
  for (const el of document.head.querySelectorAll('link[rel="icon"]')) { el.remove() }
  document.head.append(favicon)
}

onMounted(() => {
  if (import.meta.client) {
    const darkMedia = globalThis.matchMedia('(prefers-color-scheme: dark)')
    setFavicon(darkMedia.matches)

    darkMedia.addEventListener('change', (e) => {
      setFavicon(e.matches)
    })

    nextTick(() => {
      const body = document.querySelector('body') as HTMLElement
      if (body) {
        body.style.transition = 'opacity 0.3s ease-in-out'
        body.style.opacity = '1'
      }
    })
  }
})
</script>

<template>
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
  <b-toast-orchestrator />
</template>
