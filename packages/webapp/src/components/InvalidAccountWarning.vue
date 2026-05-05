<script lang="ts" setup>
const store = useClientStore()
const { publicKey } = useWalletComposable()
const isValidAccount = computed(() => store.isValidAccount && !!publicKey)
</script>

<template>
  <div class="container">
    <warning-block
      v-if="!isValidAccount && publicKey"
      :text="`The wallet address ${shortenAddress(publicKey)} is not active on the network. Please fund your account to continue.`"
    />
  </div>
</template>

<style lang="scss" scoped>
.warning-block {
  margin-top: $spacing-xl;
}
</style>
