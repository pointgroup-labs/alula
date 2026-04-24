<script lang="ts" setup>
import { PasswordProtect, usePasswordProtect } from '~/features/password-protect'

const { isNeedLogin } = usePasswordProtect()

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
  <password-protect v-if="isNeedLogin" />
  <NuxtLayout v-else>
    <NuxtPage />
  </NuxtLayout>
  <b-toast-orchestrator />
</template>
