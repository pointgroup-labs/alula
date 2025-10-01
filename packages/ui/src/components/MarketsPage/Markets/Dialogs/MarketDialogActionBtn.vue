<script lang="ts" setup>
// import type { Pool } from '@jlend/sdk'
import type { BButtonProps } from 'bootstrap-vue-next'
import { clickElement, destructurePoolAsset, shortenAddress } from '~/utils'

const {
  loading = false,
  variant = 'primary',
  pool,
  ...props
} = defineProps<{
  pool?: any
  isTrust?: boolean
  loading?: boolean
  variant?: 'primary' | 'accent'
} & BButtonProps>()

const emit = defineEmits(['clickHandler', 'closeModal'])

const toast = useToast()
const { generateExplorerLink } = useExplorerLink()

const txLoading = ref(false)
const isLoading = computed(() => txLoading.value || loading)

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const market = useMarketActions()

const assetData = computed(() => destructurePoolAsset(String(pool?.name)) || [])

const isTrust = computed(() => {
  const asset_issuer = assetData.value?.[1]
  return pool?.token_ticker === 'XLM'
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
    :variant="variant"
    :loading="isLoading"
    v-bind="props"
    pill
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
