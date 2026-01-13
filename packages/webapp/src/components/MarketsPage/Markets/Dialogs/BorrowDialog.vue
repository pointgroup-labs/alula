<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { calcFee } from '@alula/client-sdk/src/utils'
import { CLEAR_DIALOG_TIMEOUT, POOL_REMAINING_BALANCE, RELOAD_FEE_INTERVAL } from '~/config'
import { focusInput, shortenNumber, truncatePercent } from '~/utils'

const props = defineProps<{ data?: MarketTableItem }>()

const marketsStore = useMarketsStore()
const market = useMarketActions()

const userStore = useUserStore()

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const poolData = toRef(props, 'data')

const {
  marketClient,
  agree,
  isLoading,
  reloadFee,
  txFee,
  poolBorrowLimit,
  availableToBorrow,
  maxLtv,
  closeLTV,
  liquidationPenalty,
  isCanBorrow,
} = useBorrowDialog(poolData)

const amount = toRef(market, 'borrowAmount')

const dialog = defineModel({ default: false })

const healthFactor = computed(() => {
  const depositUsd = userStore.userTotalDepositInUsd
  const borrowedUsd = userStore.userTotalBorrowedInUsd
  const price = poolData.value?.price || 0
  const closeLTV = Number(poolData.value?.raw.pool.config.health_config.close_ltv_bps || 0) / 10_000

  const extraBorrowUsd = (amount.value || 0) * price
  const totalBorrowUsd = borrowedUsd + extraBorrowUsd

  let hf = (depositUsd * closeLTV) / totalBorrowUsd

  if (!Number.isFinite(hf)) {
    hf = 0
  }

  return Math.min(hf, 10)
})

const marketFee = computed(() => {
  const marketFeeBps = poolData.value?.raw.pool.config.fee_config.borrow_fee_bps
  return calcFee(Number(amount.value || 0), marketFeeBps || 0)
})

const attentionText = computed(() =>
  isCanBorrow.value
    ? 'Parameter changes via governance can alter your account health factor and risk of liquidation.'
    : 'You cannot open a loan in the same pool where you have a deposit.')

async function borrow() {
  if (!publicKey.value || !poolData.value?.raw.pool.pool_address) {
    return
  }
  if (!amount.value || amount.value <= 0) {
    focusInput('.borrow-dialog')
    return
  }

  try {
    marketsStore.poolActiveAddress = poolData.value?.raw.pool.pool_address

    const marketProps = {
      market: marketsStore.selectedMarketName,
      client: marketClient.value!,
      pool_address: poolData.value?.raw.pool.pool_address,
      amount: amount.value,
      asset_data: poolData.value?.raw.pool.name,
      poolBorrowLimit: poolBorrowLimit.value,
    }

    await market.borrow(marketProps)
  } finally {
    marketsStore.poolActiveAddress = undefined
  }
}

let interval: string | number | NodeJS.Timeout | undefined

watch(dialog, async (v) => {
  clearInterval(interval)
  if (!v) {
    setTimeout(() => {
      amount.value = 0
    }, CLEAR_DIALOG_TIMEOUT)
    agree.value = false
    return
  }

  interval = setInterval(() => {
    reloadFee.value = true
    nextTick(() => {
      reloadFee.value = false
    })
  }, RELOAD_FEE_INTERVAL)
})
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="supply-dialog borrow-dialog dialog-default"
  >
    <template #header>
      <div class="supply-dialog__title">
        <img
          :src="data?.asset.icon"
          :alt="`${data?.asset.symbol} icon`"
        >
        <span>Borrow {{ data?.asset.symbol }}</span>
      </div>
    </template>

    <div class="supply-dialog__body">
      <input-widget
        v-model="amount"
        :balance="availableToBorrow"
        :fee="POOL_REMAINING_BALANCE"
        :rules="[
          (v: any) => {
            return v && Number(v) < availableToBorrow || 'Borrow limit exceeded'
          },
        ]"
      >
        <template #label-right>
          Available: {{ formatPrice(availableToBorrow, 0, 7) }} {{ data?.asset.symbol }}
        </template>
      </input-widget>

      <div
        v-if="data"
        class="dialog-info-table"
      >
        <!-- Health Factor -->
        <div class="dialog-info-table__item">
          <span>Health Factor</span>
          <span>
            <template v-if="isLoading">
              <j-loading-spinner
                width="14px"
                style="padding: 0; width: 14px; margin-left: auto"
              />
            </template>
            <template v-else>
              {{ truncatePercent(healthFactor) }}
            </template>
          </span>
        </div>

        <!-- Pool available -->
        <div class="dialog-info-table__item">
          <span>Pool Liquidity Available</span>
          <span>
            {{ shortenNumber(poolBorrowLimit || 0) }}
          </span>
        </div>

        <!-- User available -->
        <div class="dialog-info-table__item">
          <span>Your Borrowing Capacity</span>
          <span>
            {{ shortenNumber(availableToBorrow || 0) }}
          </span>
        </div>

        <!-- Max LTV -->
        <div class="dialog-info-table__item">
          <span>Open LTV</span>
          <span>
            {{ maxLtv }}
          </span>
        </div>

        <!-- Liquidation LTV -->
        <div class="dialog-info-table__item">
          <span>Close LTV</span>
          <span>
            {{ truncatePercent(closeLTV || 0, 2) }}%
          </span>
        </div>

        <!-- Liquidation penalty -->
        <div class="dialog-info-table__item">
          <span>Liquidation Penalty</span>
          <span>
            {{ truncatePercent(liquidationPenalty || 0, 2) }}%
          </span>
        </div>

        <!-- Market fee -->
        <div class="dialog-info-table__item">
          <span>Operation Fee</span>
          <span>
            {{ formatPrice(marketFee, 0, 5) }} {{ data?.asset.symbol }}
          </span>
        </div>

        <!-- Tx fee -->
        <div class="dialog-info-table__item">
          <span>Transaction Fee</span>
          <span>
            {{ txFee }}
          </span>
        </div>
      </div>

      <warning-block
        :text="attentionText"
        :is-warning="!isCanBorrow"
      />

      <div class="supply-agree">
        <j-checkbox
          v-model="agree"
          :disabled="!isCanBorrow"
        >
          I acknowledge the risks involved.
        </j-checkbox>
      </div>

      <div class="supply-dialog-action">
        <div class="action-info">
          <span>Borrow APY</span>
          <span>{{ data?.borrow_apy }}</span>
        </div>

        <market-dialog-action-btn
          variant="accent"
          :loading="isLoading"
          :pool="data?.raw.pool"
          :disabled="!agree || !isCanBorrow"
          @click-handler="borrow"
        >
          Borrow {{ data?.asset.symbol }}
        </market-dialog-action-btn>
      </div>
    </div>
  </j-dialog>
</template>

<style lang="scss">
.borrow-dialog {
  .supply-agree {
    font-size: 11px;
    font-style: normal;
    font-weight: 500;
    line-height: 12px;
    display: flex;
    align-items: center;
    gap: $spacing-8;
  }
}
</style>
