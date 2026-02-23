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
  closeLTV,
  liquidationPenalty,
  isCanBorrow,
  attentionText,
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

async function borrow() {
  if (!publicKey.value || !poolData.value?.raw.pool.pool_address) {
    return
  }
  if (!amount.value || amount.value <= 0) {
    focusInput('.borrow-input')
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

    marketsStore.dialogBorrow = false
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
    class-name="supply-dialog dialog-default"
  >
    <template #header>
      <div class="supply-dialog__title">
        <img
          :src="data?.asset.icon"
          :alt="`${data?.asset.symbol} icon`"
        >
        <span>Borrow {{ data?.asset.symbol }}</span>
      </div>

      <div class="dialog-balance">
        <div class="dialog-balance__label">Available:</div>
        <div class="dialog-balance__value">{{ shortenNumber(availableToBorrow) }} {{ data?.asset.symbol }}</div>
      </div>
    </template>

    <div class="supply-dialog__body">
      <input-widget
        v-model="amount"
        class="borrow-input"
        :balance="availableToBorrow"
        :fee="POOL_REMAINING_BALANCE"
        :rules="[
          (v: any) => {
            return v && Number(v) < availableToBorrow || 'Borrow limit exceeded'
          },
        ]"
      />

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
            {{ poolData?.open_ltv }}
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

        <div class="separator" />
      </div>

      <warning-block
        :text="attentionText"
        :is-warning="!isCanBorrow"
      />

      <div class="extra-info">
        <j-checkbox
          v-model="agree"
          :disabled="!isCanBorrow"
        >
          <div class="extra-info__label">
            I acknowledge the risks involved.
          </div>
        </j-checkbox>
      </div>

      <div class="extra-info">
        <div class="extra-info__label">Borrow APY</div>
        <div class="extra-info__value">{{ data?.borrow_apy }}</div>
      </div>

      <div class="supply-dialog-action">

        <market-dialog-action-btn
          variant="accent"
          :loading="isLoading"
          :pool="data?.raw.pool"
          :disabled="!agree || !isCanBorrow || amount > availableToBorrow"
          @click-handler="borrow"
        >
          Borrow {{ data?.asset.symbol }}
        </market-dialog-action-btn>
      </div>
    </div>
  </j-dialog>
</template>
