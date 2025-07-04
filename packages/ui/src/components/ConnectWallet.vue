<script lang="ts" setup>
import { shortenAddress } from '~/utils'

const connection = useConnectionStore()
const wallet = useWallet()
const loading = computed(() => connection.loading)
const publicKey = computed(() => wallet.publicKey)

const Toast = useToast()

const connectWallet = async () => {
  await connection.connectWallet()
}

function disconnect() {
  connection.disconnect()
}

function copy() {
  navigator.clipboard.writeText(String(publicKey.value))
  Toast.create({
    body: `Copied Address`,
    variant: 'info',
  })
}
</script>

<template>
  <j-btn
    v-if="!publicKey"
    :loading="loading"
    pill
    size="lg"
    class="connect-wallet"
    @click="connectWallet"
  >
    Connect Wallet
  </j-btn>
  <j-popover
    v-else
    position="bottom"
    class="wallet-popover"
    close-popup
  >
    <div
      class="wallet-popover__item"
      @click="copy"
    >
      Copy Address <i-app-copy :color="isDark ? '#fff' : '#878787'" />
    </div>
    <div
      class="wallet-popover__item"
      @click="disconnect"
    >
      Disconnect
    </div>
    <template #target>
      <j-btn
        :loading="loading"
        pill
        size="lg"
        class="connect-wallet"
      >
        {{ shortenAddress(publicKey) }}
      </j-btn>
    </template>
  </j-popover>
</template>

<style lang="scss">
.wallet-popover {
  .popover-body {
    padding-left: 0;
    padding-right: 0;
  }

  &__item {
    cursor: pointer;
    padding: $spacing-8 $spacing-16;
    display: flex;
    align-items: center;
    gap: $spacing-8;

    &:hover {
      background-color: $neutral-5;
    }
  }
}
</style>
