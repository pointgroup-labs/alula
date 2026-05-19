<script lang="ts" setup>
import { capitalize } from 'vue'

const { publicKey } = useWalletComposable()
const connection = useConnectionStore()

const walletName = computed(() => {
  return capitalize(connection.selectedWalletId || '-')
})

function disconnect() {
  connection.disconnect()
}
</script>

<template>
  <div
    class="setting-item connect"
  >
    <div
      v-if="publicKey"
      class="wallet-info"
    >
      <div class="wallet-info__details">
        {{ shortenAddress(String(publicKey), 8) }}
        <copy-to-clipboard
          :text="publicKey"
          entity="address"
          color="#fff"
        />
        <j-tooltip>
          <i-app-disconnect-icon
            class="disconnect-icon"
            @click="disconnect"
          />
          <template #content>
            Disconnect
          </template>
        </j-tooltip>
      </div>

      <div class="wallet-info__name">
        {{ walletName }}
      </div>
    </div>
    <connect-wallet v-else />
  </div>
</template>

<style lang="scss">
.setting-item.connect {
  .connect-wallet {
    width: 100%;
  }

  .wallet-info {
    color: $text-primary;
    height: 46px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 14px;
    font-style: normal;
    font-weight: 700;
    line-height: 20px;

    &__details {
      font-size: 14px;
      font-style: normal;
      font-weight: 700;
      line-height: 20px;
      display: flex;
      align-items: center;
      gap: 8px;

      .disconnect-icon {
        margin-left: $spacing-xs;
        cursor: pointer;
      }
    }

    &__name {
      color: rgba($text-secondary, 0.4);
      font-size: 12px;
      font-style: normal;
      font-weight: 400;
      line-height: 14px;
    }
  }
}
</style>
