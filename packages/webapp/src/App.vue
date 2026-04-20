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

// @ts-expect-error...
const config = useRuntimeConfig().public

useSeoMeta({
  title: config.APP_TITLE ?? 'Alula Protocol | Institution-Ready Credit Layer on Stellar',
  description: config.APP_DESCRIPTION ?? 'Access secure RWA lending pools and high-yield vaults on Stellar. Supply USDC/XLM or borrow against tokenized assets with institution-grade risk controls.',
  keywords: config.KEYWORDS ?? 'Alula, DeFi lending, RWA investment, Stellar stablecoins, USDC yield, institutional DeFi, tokenized credit, Soroban, lending, borrowing, rwa lending',
  ogType: 'website',
  ogUrl: config.APP_URL,
  ogTitle: config.OG_TITLE ?? 'Alula Dashboard | Institutional Yield & Credit',
  ogDescription: config.OG_DESCRIPTION ?? 'Manage your DeFi position with Cross-Pool Evaluation and JIT liquidity. Earn yield on stablecoins or access compliant RWA-backed credit lines.',
  ogImage: `${config.APP_URL}/og-image-1200x630.png`,
  twitterCard: 'summary_large_image',
  twitterTitle: config.TWITTER_TITLE ?? 'Alula Protocol Dashboard | Institutional Yield & Credit',
  twitterDescription: config.TWITTER_DESCRIPTION ?? 'Optimized RWA lending on Stellar. Access permissioned pools, fixed-rate credit, and secure yield orchestrators.',
  twitterImage: `${config.APP_URL}/og-image-1200x630.png`,
})
</script>

<template>
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
  <b-toast-orchestrator />
</template>
