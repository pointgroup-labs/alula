<script lang="ts" setup>
const { width } = useWindowSize()
const store = useClientStore()
const walletStore = useWallet()
const isValidAccount = computed(() => store.isValidAccount && !!walletStore.publicKey)
</script>

<template>
  <main class="page-container container">
    <warning-block
      v-if="!isValidAccount && walletStore.publicKey"
      text="The wallet address GA726GBW...SAWNEACK does not exist on the network. Please fund your account!"
    />
    <markets-info v-if="width > 1024" />
    <markets />
  </main>
</template>
