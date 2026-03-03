<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { calcFee, calcUserTotalBorrowedInUsd, calcUserTotalStakeInUsd } from '@alula/client-sdk/src/utils'
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
  const marketName = String(poolData.value?.market)
  const obligation = userStore.state.obligations[marketName]
  const marketState = marketsStore.state.markets[marketName]?.marketState
  if (!obligation || !marketState) {
    return 0
  }
  const assetDecimals = marketState.asset_decimals ?? 7
  const oraclePriceDecimals = marketState.oracle_price_decimals ?? 0
  const poolsData = marketState.pools_data

  const depositUsd = calcUserTotalStakeInUsd(obligation, poolsData, assetDecimals, oraclePriceDecimals, 'open')
  const borrowedUsd = calcUserTotalBorrowedInUsd(obligation, poolsData, assetDecimals, oraclePriceDecimals) ?? 0
  const price = poolData.value?.price || 0

  const extraBorrowUsd = (amount.value || 0) * price
  const totalBorrowUsd = borrowedUsd + extraBorrowUsd

  const hf = totalBorrowUsd > 0 ? depositUsd / totalBorrowUsd : 0

  return Math.min(hf, 10)
})

const marketFee = computed(() => {
  const marketFeeBps = poolData.value?.raw.pool.config.fee_config.borrow_fee_bps
  return calcFee(Number(amount.value || 0), marketFeeBps || 0)
})

const infoPanelData = computed(() => {
  if (!poolData.value) {
    return {}
  }
  return {
    poolInfo: {
      title: 'Pool Info',
      data: [
        {
          label: 'Pool Liquidity Available',
          value: shortenNumber(poolBorrowLimit.value || 0),
        },
        {
          label: 'Open LTV',
          value: poolData.value.open_ltv,
        },
        {
          label: 'Close LTV',
          value: truncatePercent(closeLTV.value || 0, 2),
        },
      ],
    },
    health: {
      title: 'Health',
      data: [
        {
          label: 'Health Factor',
          value: truncatePercent(healthFactor.value),
          slotName: 'hf',
        },
        {
          label: 'Borrowing Capacity',
          value: shortenNumber(availableToBorrow.value || 0),
        },
        {
          label: 'Liquidation Penalty',
          value: truncatePercent(liquidationPenalty.value || 0, 2),
        },
      ],
    },
    fees: {
      title: 'Operation Fee',
      data: [
        {
          label: 'Operation Fee',
          value: `${formatPrice(marketFee.value, 0, 5)} ${poolData.value?.asset.symbol}`,
        },
        {
          label: 'Transaction Fee',
          value: `${txFee.value} ${poolData.value?.asset.symbol}`,
        },
      ],
    },
  }
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
      <div class="dialog-default__title">
        <img
          :src="data?.asset.icon"
          :alt="`${data?.asset.symbol} icon`"
        >
        <span>Borrow {{ data?.asset.symbol }}</span>
      </div>
    </template>

    <div class="dialog-default__body">
      <input-widget
        v-model="amount"
        class="borrow-input mb-2"
        :balance="availableToBorrow"
        :fee="POOL_REMAINING_BALANCE"
        label-left="Available"
        :label-right="`${formatPrice(availableToBorrow ?? 0, 0, 4)} ${data?.asset.symbol}`"
        :price="poolData?.price"
        :reset="dialog"
        variant="borrow"
        :rules="[
          (v: any) => {
            return Number(v) < availableToBorrow || 'Borrow limit exceeded'
          },
        ]"
      />

      <template v-if="data">
        <!-- Pool info -->
        <info-panel
          :title="infoPanelData.poolInfo!.title"
          :data="infoPanelData.poolInfo!.data"
          variant="borrow"
        />

        <!-- Health -->
        <info-panel
          :title="infoPanelData.health!.title"
          :data="infoPanelData.health!.data"
          variant="borrow"
        >
          <template #hf>
            <j-loading-spinner
              v-if="isLoading"
              width="14px"
              style="padding: 0; width: 14px;"
            />
            <template v-else>
              {{ truncatePercent(healthFactor) }}
            </template>
          </template>
        </info-panel>

        <!-- Fees -->
        <info-panel
          :title="infoPanelData.fees!.title"
          :data="infoPanelData.fees!.data"
          variant="borrow"
        />
      </template>

      <warning-block
        :text="attentionText"
        :is-warning="!isCanBorrow"
      />

      <div class="extra-info">
        <j-checkbox
          v-model="agree"
          :disabled="!isCanBorrow"
          color="#6366F1"
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

      <div class="dialog-default__action mt-2">
        <market-dialog-action-btn
          variant="purple"
          pill
          size="lg"
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
