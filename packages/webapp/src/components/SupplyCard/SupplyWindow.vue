<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { calcFee } from '@alula/client-sdk/src/utils'
import { POOL_REMAINING_BALANCE } from '~/config'
import { focusInput, formatPrice } from '~/utils'

const emits = defineEmits(['dialogHandler'])

const selectedPool = inject<Ref<MarketTableItem>>('selectedPool')

const marketsStore = useMarketsStore()
const market = useMarketActions()

const amount = toRef(market, 'depositAmount')

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const {
  marketClient,
  collateralOnly,
  balance,
  txFee,
  isLoadingFee,
  supplyLimit,
  limitLabel,
  isLoading,
  isCanSupply,
  attentionText,
} = useSupplyDialog(selectedPool)

const marketFee = computed(() => {
  const marketFeeBps = collateralOnly.value
    ? selectedPool?.value?.raw.pool.config.fee_config.add_collateral_fee_bps
    : selectedPool?.value?.raw.pool.config.fee_config.deposit_fee_bps
  return calcFee(Number(amount.value || 0), marketFeeBps || 0)
})

const reserveAmount = computed(() => selectedPool?.value?.raw.pool.token_symbol === 'native' ? 2 : 0)

async function supply() {
  try {
    if (!publicKey.value || !selectedPool?.value?.raw.pool.pool_address) {
      return
    }
    if (!amount.value || amount.value <= 0) {
      focusInput('.input-wrapper')
      return
    }
    marketsStore.poolActiveAddress = selectedPool?.value?.raw.pool.pool_address

    const marketProps = {
      market: marketsStore.selectedMarketName,
      client: marketClient.value!,
      pool_address: selectedPool?.value?.raw.pool.pool_address,
      amount: amount.value,
      asset_data: selectedPool?.value?.raw.pool.name,
    }
    collateralOnly.value
      ? await market.addCollateral(marketProps)
      : await market.deposit(marketProps)

    marketsStore.dialogSupply = false
  } finally {
    marketsStore.poolActiveAddress = undefined
  }
}

const debouncedFn = useDebounceFn(calculateRewardsEarnings, 500)

function calculateRewardsEarnings(
  deposit: number,
  apyPercent: number,
  price: number,
) {
  const apy = apyPercent / 100
  const dailyRate = (1 + apy) ** (1 / 365) - 1
  const daily = deposit * dailyRate * price
  const yearly = deposit * apy * price
  return {
    daily: daily.toFixed(daily > 1 ? 2 : 4),
    yearly: yearly.toFixed(yearly > 1 ? 2 : 4),
  }
}

const rewardsEarnings = computedAsync(async () => {
  if (!amount.value || amount.value === 0) {
    return {
      daily: 0,
      yearly: 0,
    }
  }
  const apyRaw = selectedPool?.value?.deposit_apy ?? '0'
  const apy = Number(apyRaw.replace('%', ''))
  const price = selectedPool?.value.price ?? 0
  return debouncedFn(
    Number(amount.value),
    apy,
    price)
})
</script>

<template>
  <div class="input-wrappe mt-4">
    <input-widget
      v-model="amount"
      :balance="balance"
      :limit="Number(supplyLimit) || 0"
      :fee="POOL_REMAINING_BALANCE + txFee + reserveAmount"
      :price="selectedPool?.price"
      label-left="You Supply"
      :label-right="`${formatPrice(balance ?? 0, 0, 4)} ${selectedPool?.asset.symbol}`"
      :rules="[
        (v) => {
          return Number(v) < balance || 'Insufficient balance'
        },
        (v) => {
          return (supplyLimit <= 0 || Number(v) <= supplyLimit) || 'Pool supply limit'
        },
      ]"
    >
      <template #prepend>
        <div
          class="select-pool-btn"
          @click="emits('dialogHandler')"
        >
          <img
            :src="selectedPool?.asset.icon"
            alt="asset icon"
          >
          {{ selectedPool?.asset.symbol }}
          <i-app-chevron-down />
        </div>
      </template>
    </input-widget>
  </div>

  <div class="collateral mt-3">
    <div class="collateral-label">Collateral Only</div>

    <j-toggle
      v-model="collateralOnly"
      size="small"
      :disabled="!isCanSupply"
      color="#22d3ee"
    />
  </div>

  <div
    class="info-card mt-3 info-supply"
    :style="{ '--color': '#22d3ee', '--bg-color': 'rgba(0, 211, 238, 0.04)', '--border-color': 'rgba(0, 211, 238, 0.1)' }"
  >
    <div class="info-supply__header">
      <div class="info-title">
        Supply APY
      </div>
      <div class="info-apy">
        {{ selectedPool?.deposit_apy }}
      </div>
    </div>
    <div class="info-supply__body">
      <div class="info-detail">
        <div class="info-detail__title">
          Daily
        </div>
        <div class="info-detail__value">
          {{ rewardsEarnings?.daily ? `$${formatPrice(rewardsEarnings?.daily)}` : '--' }}
        </div>
      </div>
      <div class="info-detail">
        <div class="info-detail__title">
          Est. Earnings / yr
        </div>
        <div class="info-detail__value">
          {{ rewardsEarnings?.yearly ? `$${formatPrice(rewardsEarnings?.yearly)}` : '--' }}
        </div>
      </div>
    </div>
  </div>

  <Transition name="summary-slide">
    <div
      v-if="amount && amount > 0 && selectedPool"
      class="info-card mt-3 info-summary"
    >
      <div class="info-summary__header">
        Transaction Summary
      </div>

      <div class="summary-list">
        <!-- Supply Limit -->
        <div class="summary-list__item">
          <div class="label">
            Supply Limit
          </div>
          <div class="value">
            {{ limitLabel }} {{ limitLabel !== '-' ? selectedPool?.asset.symbol : '' }}
          </div>
        </div>

        <!-- Open LTV -->
        <div class="summary-list__item">
          <div class="label">
            Open LTV
          </div>
          <div class="value">
            {{ selectedPool?.open_ltv }}
          </div>
        </div>

        <!-- Utilization Rate -->
        <div class="summary-list__item">
          <div class="label">
            Utilization Rate
          </div>
          <div class="value">
            {{ selectedPool?.utilization_rate }}
          </div>
        </div>

        <!-- Operation Fee -->
        <div class="summary-list__item">
          <div class="label">
            Operation Fee
          </div>
          <div class="value">
            {{ formatPrice(marketFee) }} XLM
          </div>
        </div>

        <!-- Transaction Fee -->
        <div class="summary-list__item">
          <div class="label">
            Transaction Fee
          </div>
          <div class="value">
            <j-loading-spinner
              v-if="isLoadingFee"
              width="14px"
              style="margin:0 20px 0 auto;"
            />
            <span v-else>{{ txFee }} XLM</span>
          </div>
        </div>
      </div>
    </div>
  </Transition>

  <warning-block
    v-if="!isCanSupply"
    :text="attentionText"
    :is-warning="!isCanSupply"
    class="mt-3"
  />

  <div class="supply-card__action mt-4">
    <market-dialog-action-btn
      variant="blue"
      size="lg"
      :loading="isLoading"
      :pool="selectedPool?.raw.pool"
      :disabled="!isCanSupply || amount >= balance"
      pill
      @click-handler="supply"
    >
      <i-metrics-complete class="complete-icon" /> Supply {{ selectedPool?.asset.symbol }}
    </market-dialog-action-btn>
  </div>
</template>
