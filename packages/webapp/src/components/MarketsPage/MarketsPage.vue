<script lang="ts" setup>
const store = useClientStore()
const { publicKey } = useWalletComposable()
const isValidAccount = computed(() => store.isValidAccount && !!publicKey)
</script>

<template>
  <main class="page-container container">
    <warning-block
      v-if="!isValidAccount && publicKey"
      :text="`The wallet address ${shortenAddress(publicKey)} does not exist on the network. Please fund your account!`"
    />
    <markets-info />
    <markets />
  </main>
</template>
