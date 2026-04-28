<script lang="ts" setup>
import type { BButtonProps } from 'bootstrap-vue-next'
import { shortenAddress } from '~/utils'

defineProps<{
} & BButtonProps>()

const connection = useConnectionStore()
const { publicKey } = useWalletComposable()
const loading = computed(() => connection.loading || connection.autoConnecting)

const toast = useToast()

const connectWallet = async () => {
  await connection.connectWallet()
}

function disconnect() {
  connection.disconnect()
}

function copy() {
  navigator.clipboard.writeText(String(publicKey.value))
  toast.create({
    body: `Copied Address`,
    variant: 'info',
  })
}
</script>

<template>
  <j-btn
    v-if="!publicKey"
    :loading="loading"
    size="sm"
    class="connect-wallet"
    variant="ghost"
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
      Copy Address <i-app-copy />
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
        size="sm"
        class="connect-wallet"
        variant="outlined-brand"
      >
        <address-icon :address="publicKey" /> {{ shortenAddress(publicKey) }}
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
    color: $text-tertiary;
    padding: $spacing-md $spacing-xl;
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;

    &:hover {
      color: $text-primary;

      svg {
        color: $text-primary;
      }
    }

    svg {
      color: $text-tertiary;
    }
  }
}
</style>
