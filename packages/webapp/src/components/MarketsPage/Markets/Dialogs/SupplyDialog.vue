<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { calcFee } from '@alula/client-sdk/src/utils'
import { CLEAR_DIALOG_TIMEOUT, POOL_REMAINING_BALANCE, RELOAD_FEE_INTERVAL } from '~/config'
import { focusInput, formatPrice } from '~/utils'

const {
  data,
} = defineProps<{
  data?: MarketTableItem
}>()

const router = useRouter()
const route = useRoute()

const dialog = defineModel({ default: false })

const loadingFee = ref(false)

const { generateExplorerLink } = useExplorerLink()

const marketsStore = useMarketsStore()
const market = useMarketActions()

const userStore = useUserStore()

const amount = toRef(market, 'depositAmount')
const collateralOnly = toRef(market, 'collateralOnly')

const marketClient = computed(() => marketsStore.marketClient)

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const balance = computed(() => {
  if (!data) {
    return 0
  }
  if (data.raw.pool.token_symbol === 'native') {
    return wallet.nativeBalance
  }
  const [, asset_issuer] = destructurePoolAsset(data?.raw.pool.name)
  return wallet.getAssetBalance(String(asset_issuer))
})

const loading = computed(() => marketsStore.poolActiveAddress === data?.raw.pool.pool_address)
const reloadFee = ref(false)

const txFee = ref(0)

const isSupplyLimited = computed(() => data?.supply_limit && data?.supply_limit > 0)
const supplyLimit = computed(() => isSupplyLimited.value ? Math.max(Number(data?.supply_limit) || 0 - Number(data?.total_supply), 0) : 0)
const limitLabel = computed(() => isSupplyLimited.value ? formatPrice(Number(data?.supply_limit) || 0, 2, 2) : '-')

const contractAddress = computed(() => data?.raw.pool.pool_address || '')

const marketFee = computed(() => {
  const marketFeeBps = collateralOnly.value ? data?.raw.pool.config.fee_config.add_collateral_fee_bps : data?.raw.pool.config.fee_config.deposit_fee_bps
  return calcFee(Number(amount.value || 0), marketFeeBps || 0)
})

const isCanSupply = computed(() => {
  const depositObligations = userStore.state.obligations[String(data?.market)]?.borrows ?? []
  for (const [address] of depositObligations) {
    if (address === data?.raw.pool.pool_address) {
      return false
    }
  }
  return true
})

async function supply() {
  try {
    if (!publicKey.value || !data?.raw.pool.pool_address) {
      return
    }
    if (!amount.value || amount.value <= 0) {
      focusInput('.supply-dialog__input')
      return
    }
    marketsStore.poolActiveAddress = data?.raw.pool.pool_address

    const marketProps = {
      market: marketsStore.activeMarketFilter,
      client: marketClient.value!,
      pool_address: data?.raw.pool.pool_address,
      amount: amount.value,
      asset_data: data?.raw.pool.name,
    }
    collateralOnly.value
      ? await market.addCollateral(marketProps)
      : await market.deposit(marketProps)
  } finally {
    marketsStore.poolActiveAddress = undefined
  }
}

watchDebounced([
  () => data,
  reloadFee,
  publicKey,
], async ([d, _r]) => {
  try {
    loadingFee.value = true

    if (!d || !publicKey.value || !marketClient.value) {
      return
    }

    const tx = await marketClient.value.marketSdk.depositTx(
      publicKey.value,
      d?.raw.pool.pool_address || '',
      0,
    )
    txFee.value = marketClient.value.marketSdk.getTransactionFee(tx)
  } finally {
    loadingFee.value = false
  }
}, { immediate: true, debounce: 300 })

let interval: string | number | NodeJS.Timeout | undefined

watch(dialog, async (v) => {
  clearInterval(interval)
  if (!v) {
    setTimeout(() => {
      amount.value = 0
    }, CLEAR_DIALOG_TIMEOUT)
    collateralOnly.value = false
    return
  }

  interval = setInterval(() => {
    reloadFee.value = true
    nextTick(() => {
      reloadFee.value = false
    })
  }, RELOAD_FEE_INTERVAL)
})

watchDebounced(collateralOnly, (c) => {
  const query = { ...route.query }
  if (c) {
    query['collateral-only'] = 'true'
  } else {
    delete query['collateral-only']
  }
  router.replace({ query })
}, { debounce: 100 })

watch(() => route.query, (q) => {
  if (q['collateral-only']) {
    collateralOnly.value = true
  }
}, { immediate: true, once: true })
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="supply-dialog dialog-default"
  >
    <template #header>
      <div class="supply-dialog__title">
        <img
          :src="data?.asset.icon"
          :alt="`${data?.asset.symbol} icon`"
        >
        <span>Supply {{ data?.asset.symbol }}</span>
      </div>
    </template>

    <div class="supply-dialog__body">
      <input-widget
        v-model="amount"
        :balance="balance"
        :limit="Number(supplyLimit) || 0"
        :fee="POOL_REMAINING_BALANCE + txFee"
        class="supply-dialog__input"
        :rules="[
          (v) => {
            return v && Number(v) < balance || 'Insufficient balance'
          },
          (v) => {
            return (supplyLimit <= 0 || Number(v) <= supplyLimit) || 'Pool supply limit'
          },
        ]"
      >
        <template #label-right>
          Balance: {{ balance }} {{ data?.asset.symbol }}
        </template>
      </input-widget>

      <div
        v-if="data"
        class="dialog-info-table"
      >
        <!-- Supply Limit -->
        <div
          class="dialog-info-table__item"
        >
          <span>Supply Limit</span>
          <span>{{ limitLabel }}</span>
        </div>

        <!-- Contract Address -->
        <div
          class="dialog-info-table__item"
        >
          <span>Contract</span>
          <a
            :href="generateExplorerLink(String(contractAddress), 'contract')"
            target="_blank"
          >{{ shortenAddress(String(contractAddress), 5) }}
            <i-app-export-icon />
          </a>
        </div>

        <!-- Market Fee -->
        <div
          class="dialog-info-table__item"
        >
          <span>Market Fee</span>

          <span>{{ formatPrice(marketFee) }} {{ data?.asset.symbol }}</span>
        </div>

        <!-- Transaction Fee -->
        <div
          class="dialog-info-table__item"
        >
          <span>Transaction Fee</span>
          <j-loading-spinner
            v-if="loadingFee"
            width="14px"
            style="margin:0 20px 0 auto;"
          />
          <span v-else>{{ txFee }} XLM</span>
        </div>
      </div>

      <warning-block
        v-if="!isCanSupply"
        text="You cannot deposit funds into a pool where you have an active loan."
        :is-warning="!isCanSupply"
      />

      <j-toggle
        v-model="collateralOnly"
        size="small"
        :disabled="!isCanSupply"
      >
        <template #append>
          Collateral Only
        </template>
      </j-toggle>

      <div class="supply-dialog-action">
        <div class="action-info">
          <span>Supply APY</span>
          <span>{{ data?.deposit_apy }}</span>
        </div>

        <market-dialog-action-btn
          variant="primary"
          :loading="loading"
          :pool="data?.raw.pool"
          :disabled="!isCanSupply"
          @click-handler="supply"
        >
          Supply {{ data?.asset.symbol }}
        </market-dialog-action-btn>
      </div>
    </div>
  </j-dialog>
</template>

<style lang="scss">
.supply-dialog {
  &__title {
    display: flex;
    align-items: center;
    gap: $spacing-8;
    font-size: 20px;
    font-style: normal;
    font-weight: 400;
    line-height: 20px;

    img {
      width: 40px;
      height: 40px;
      object-fit: contain;
      border-radius: 50%;
    }
  }

  &__body {
    padding-top: $spacing-16;
    display: flex;
    flex-direction: column;
    gap: $spacing-16;
  }

  .j-toggle__label {
    font-size: 14px;
    user-select: none;
  }

  .supply-dialog-action {
    display: flex;
    justify-content: space-between;
    gap: $spacing-32;

    .action-info {
      white-space: nowrap;
      flex: 1;
      display: flex;
      flex-direction: column;
      gap: 2px;

      span:first-child {
        color: $neutral-12;
        font-size: 12px;
        font-style: normal;
        font-weight: 500;
        line-height: 16px;
      }

      span:last-child {
        font-size: 20px;
        font-style: normal;
        font-weight: 700;
        line-height: 20px;
      }
    }

    .btn {
      width: 100%;
    }
  }
}

body.body--dark {
  .supply-dialog {
    .j-input .j-input__label {
      color: $neutral-12;
    }
  }
}
</style>
