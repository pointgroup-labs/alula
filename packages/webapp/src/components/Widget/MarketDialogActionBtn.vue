<script lang="ts" setup>
// import type { Pool } from '@jlend/sdk'
import type { Pool } from '@alula/market-sdk'
import type { BButtonProps } from 'bootstrap-vue-next'
import { clickElement, destructurePoolAsset, shortenAddress } from '~/utils'

const {
  loading = false,
  variant = 'blue',
  pool,
  ...props
} = defineProps<{
  pool?: Pool
  isTrust?: boolean
  loading?: boolean
  variant?: 'blue' | 'accent'
  disabled?: boolean
} & BButtonProps>()

const emit = defineEmits(['clickHandler', 'closeModal'])

const toast = useToast()
const { generateExplorerLink } = useExplorerLink()

const txLoading = ref(false)
const isLoading = computed(() => txLoading.value || loading)

const connection = useConnectionStore()
const isConnectionLoading = computed(() => connection.loading)

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const market = useMarketActions()

const assetData = computed(() => destructurePoolAsset(String(pool?.name)) || [])

const isTrust = computed(() => {
  const asset_issuer = assetData.value?.[1]
  return pool?.token_symbol === 'native'
    || !!wallet.balances?.find((b: any) => b.asset_issuer?.toLowerCase() === asset_issuer?.toLowerCase())
})

async function addTrust() {
  try {
    txLoading.value = true
    const [asset, issuer] = assetData.value
    const res = await market.addTrustLine(String(asset), String(issuer))
    toast.create({
      title: 'Add Trust Success',
      body: `You added trustline for ${asset} ${shortenAddress(String(issuer), 6)}`,
      modelValue: 30_000,
      alertProps: {
        variant: 'success',
      },
      actions: [
        {
          label: 'View Transaction',
          href: generateExplorerLink(String(res?.hash)),
          onClick: () => {
            emit('closeModal')
          },
        },
      ],
    })
  } catch (error: any) {
    const message = error?.message || String(error)
    toast.create({
      title: 'Add Trust Error',
      body: String(message),
      modelValue: 0,
      alertProps: {
        variant: 'error',
      },

    })
    throw error
  } finally {
    txLoading.value = false
  }
}

async function emitClickHandler() {
  if (!isTrust.value && publicKey.value) {
    await addTrust()
    return
  }
  if (!publicKey.value) {
    clickElement('.connect-wallet')
    emit('closeModal')
    return
  }
  emit('clickHandler')
}
</script>

<template>
  <j-btn
    :variant="publicKey ? variant : 'ghost'"
    :loading="isLoading || isConnectionLoading"
    v-bind="props"
    :disabled="!publicKey ? false : disabled"
    class="market-action-btn"
    @click="emitClickHandler"
  >
    <template v-if="!publicKey">
      Connect Wallet
    </template>
    <slot v-else-if="isTrust" />
    <template v-else>
      Add Trust
    </template>
  </j-btn>
</template>

<style lang="scss">
.market-action-btn {
  --bs-btn-active-color: #000;
  width: 100%;

  &.btn-blue {
    background-color: $supply;
    color: $dark;

    &:hover {
      background-color: lighten($supply, 10%);
    }
  }

  &.btn-accent {
    background-color: $purple;
    color: $foreground;

    &:hover {
      background-color: lighten($purple, 10%);
    }
  }

  .complete-icon {
    width: 16px;
    height: 16px;
  }
}
</style>
