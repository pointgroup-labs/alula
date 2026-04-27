<script lang="ts" setup>
import type { Pool } from '@alula/market-sdk'
import type { BButtonProps } from 'bootstrap-vue-next'
import { clickElement, destructurePoolAsset, shortenAddress } from '~/utils'

const {
  loading = false,
  variant = 'brand',
  pool,
  poolSecondary,
  ...props
} = defineProps<{
  pool?: Partial<Pool>
  poolSecondary?: Partial<Pool>
  isTrust?: boolean
  loading?: boolean
  variant?: 'brand' | 'brand-secondary' | 'positive' | 'negative'
  disabled?: boolean
} & BButtonProps>()

const emit = defineEmits(['clickHandler', 'closeModal'])

const toast = useToast()
const { generateExplorerLink } = useExplorerLink()

const txLoading = ref(false)
const isLoading = computed(() => txLoading.value || loading)

const connection = useConnectionStore()
const isConnectionLoading = computed(() => connection.loading)

const { publicKey, balances } = useWalletComposable()

const marketsStore = useMarketsStore()
const market = useMarketActions()

const requiredAssets = computed(() => {
  return [pool, poolSecondary]
    .filter((item): item is Pool => !!item)
    .map((item) => {
      const [asset, issuer] = destructurePoolAsset(String(item.name)) || []
      return {
        asset,
        issuer,
        isNative: item.token_symbol === 'native',
      }
    })
    .filter(item => item.isNative || (item.asset && item.issuer))
})

const missingAsset = computed(() => {
  return requiredAssets.value.find((assetData) => {
    if (assetData.isNative) {
      return false
    }

    return !balances.value?.find((balance: any) => balance.asset_issuer?.toLowerCase() === assetData.issuer?.toLowerCase())
  })
})

const isTrust = computed(() => {
  return !missingAsset.value
})

async function addTrust() {
  try {
    txLoading.value = true
    const assetData = missingAsset.value
    if (!assetData?.asset || !assetData.issuer) {
      return
    }

    if (!marketsStore.activeMarket) {
      const market = Object.values(marketsStore.state.markets).find((market) => {
        return market.marketState.pools_data.some((p) => {
          return p.pool.name === pool?.name
        })
      })

      if (market?.marketName) {
        marketsStore.selectedMarketName = market.marketName
      } else {
        throw new Error('Market not found')
      }
    }

    const { asset, issuer } = assetData
    const res = await market.addTrustLine(String(asset), String(issuer))
    toast.create({
      title: 'Add Trust Success',
      body: `You added trustline for ${asset} ${shortenAddress(String(issuer), 6)}`,
      modelValue: 30_000,
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
