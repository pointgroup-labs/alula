<script lang="ts" setup>
const store = useClientStore()
const walletStore = useWallet()
const isValidAccount = computed(() => store.isValidAccount && !!walletStore.publicKey)
</script>

<template>
  <div
    v-if="!isValidAccount && walletStore.publicKey"
    class="not-valid-account-warning"
  >
    <i-app-warning-color class="warning-icon" />
    <div class="warning-text">
      The wallet address {{ shortenAddress(String(walletStore.publicKey), 8) }} does not exist on the network.
      Please fund your account!
    </div>
  </div>
</template>

<style lang="scss">
.not-valid-account-warning {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 8px;
  background-color: #ffb72642;
  padding: $spacing-xl;
  border-radius: $radius-md;

  .warning-icon {
    width: 22px;
    min-width: 22px;
    height: 22px;
    min-height: 22px;
  }

  .warning-text {
    font-size: 14px;
    line-height: normal;
    color: #4e4e4e;
  }
}

.theme-dark {
  .not-valid-account-warning {
    background-color: #ffbb0042;

    .warning-text {
      color: #fff;
    }
  }
}
</style>
