<script lang="ts" setup>
import type { BButtonProps } from 'bootstrap-vue-next'
import type { Pool } from 'sdk'
import { shortenAddress } from '~/utils'

const {
  loading = false,
  variant = 'primary',
  pool,
  ...props
} = defineProps<{
  pool?: Pool
  isTrust?: boolean
  loading?: boolean
  variant?: 'primary' | 'accent'
} & BButtonProps>()

const emit = defineEmits(['clickHandler', 'closeModal'])

const Toast = useToast()

const txLoading = ref(false)
const isLoading = computed(() => txLoading.value || loading)

const marketsStore = useMarketsStore()
const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const assetData = computed(() => pool?.name.split(':') || [])

const isTrust = computed(() => {
  const asset_issuer = assetData.value?.[1]
  return pool?.token_ticker === 'XLM'
    || !!wallet.balances?.find((b: any) => b.asset_issuer?.toLowerCase() === asset_issuer?.toLowerCase())
})

async function addTrust() {
  try {
    txLoading.value = true
    const [asset, issuer] = assetData.value
    const res = await marketsStore.addTrustLine(asset, issuer)
    Toast.create({
      title: 'Add Trust Success',
      body: `You added trustline for ${asset} ${shortenAddress(issuer, 6)}`,
      modelValue: 30_000,
      alertProps: {
        variant: 'success',
      },
      actions: [
        {
          label: 'View Transaction',
          href: `https://stellar.expert/explorer/testnet/tx/${res.hash}`,
          onClick: () => {
            emit('closeModal')
          },
        },
      ],
    })
  } catch (error: any) {
    const message = error?.message || String(error)
    Toast.create({
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
    const btn = document.querySelector('.connect-wallet') as HTMLElement
    btn?.click()
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
    <slot v-if="isTrust || !publicKey" />
    <template v-else>
      Add Trust
    </template>
  </j-btn>
</template>
