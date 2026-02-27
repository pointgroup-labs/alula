<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { calcFee } from '@alula/client-sdk/src/utils'
import Decimal from 'decimal.js'
import { POOL_REMAINING_BALANCE } from '~/config'
import { focusInput, formatPrice } from '~/utils'

const selectedPool = inject<Ref<MarketTableItem>>('selectedPool')

const route = useRoute()
const router = useRouter()

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
      focusInput('.supply-dialog__input')
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

const receiveAmountInUSD = computed(() => {
  if (!amount.value || !selectedPool?.value?.price) {
    return 0
  }
  return amount.value * selectedPool?.value?.price
})

const inputErrors = computed(() => {
  if (amount.value > balance.value) {
    return 'Insufficient balance'
  }
  if (supplyLimit.value > 0 && amount.value > supplyLimit.value) {
    return 'Pool supply limit'
  }
  return ''
})

const debouncedFn = useDebounceFn((amount: number, apy: number, price: number) => calculateRewardsEarnings(amount, apy, price), 500)

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
    daily,
    yearly,
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

const selectedOption = ref()

const options = computed(() => {
  return marketsStore.selectedMarketPools?.map(({ pool }) => {
    const asset = getFullTokenData(pool.token_symbol)
    return {
      label: asset.symbol,
      value: pool.pool_address,
      icon: asset.icon,
    }
  }) ?? []
})

const selectedAmount = ref<string | null>(null)

const amountActions = ['25%', '50%', '75%', 'max']

function handleAmount(percent: string | null) {
  if (!percent) {
    return
  }

  selectedAmount.value = percent
  amount.value = max(percent.replace('%', ''))
}

function max(percent?: string | number) {
  const b = new Decimal(balance.value)
  const f = new Decimal(POOL_REMAINING_BALANCE + txFee.value + reserveAmount.value)
  const result = b.minus(f).toNumber()
  const maxVal = Math.max(Math.min(result, supplyLimit.value || balance.value), 0) || 0
  const decimals = String(maxVal).includes('e') ? getZeroCountAfterDecimal(maxVal) : null
  let maxAmount = decimals ? maxVal.toFixed(decimals) : String(maxVal)
  const [, dec] = maxAmount.toString().split('.')
  if (!decimals && dec && dec.length > market.assetDecimals.value) {
    maxAmount = truncatePercent(Number(maxAmount), market.assetDecimals.value)
  }
  if (percent && percent !== 'max') {
    return Number(maxAmount) * (Number(percent) / 100)
  }
  return maxAmount
}

watch(selectedOption, (opt) => {
  if (!opt) {
    return
  }

  router.replace({
    name: route.name as string,
    params: {
      ...route.params,
      pool: opt.value,
    },
    query: route.query,
    hash: route.hash,
  })
})

const stopRef: { stop?: () => void } = {}

stopRef.stop = watch(() => selectedPool?.value, (val) => {
  if (!options.value) {
    return
  }
  selectedOption.value = options.value.find(option => option.value === val?.pool_address)
  if (selectedOption.value) {
    stopRef.stop?.()
  }
}, { immediate: true })
</script>

<template>
  <div class="supply-card">

    <div class="supply-card__body">
      <div class="input-wrapper">
        <div class="wallet-balance">
          <div class="wallet-balance__label">You Supply</div>
          <div class="wallet-balance__value">{{ formatPrice(balance, 2, 5) }} {{ selectedPool?.asset.symbol }}</div>
        </div>

        <div
          class="input-block info-card"
          :class="{ active: amount && amount > 0, error: inputErrors }"
        >
          <div class="input-block__top">
            <j-select
              v-model="selectedOption"
              :options="options"
            >
              <template #label>
                <img
                  :src="selectedOption?.icon"
                  alt="asset icon"
                  style="width: 24px; height: 24px; margin-right: 2px;"
                >
                {{ selectedOption?.label }}
              </template>

              <template #option="{ option }">
                <img
                  v-if="option?.icon"
                  :src="option?.icon"
                  alt="asset icon"
                  style="width: 24px; height: 24px; margin-right: 2px;"
                >
                {{ option.label }}
              </template>
            </j-select>

            <j-input
              v-model="amount"
              size="sm"
              placeholder="0.00"
              only-numbers
              @keyup="selectedAmount = null"
            />
          </div>
          <div class="input-block__btns">
            <div class="select-amount">
              <span
                v-for="value in amountActions"
                :key="value"
                :class="{ active: value === selectedAmount }"
                @click="handleAmount(value)"
              >{{ value }}</span>
            </div>
            <div class="amount-to-dollar">
              ${{ formatPrice(receiveAmountInUSD, 2, 2) }}
            </div>
          </div>
        </div>

        <div
          v-if="inputErrors"
          class="input-errors"
        >
          {{ inputErrors }}
        </div>
      </div>

      <warning-block
        v-if="!isCanSupply"
        :text="attentionText"
        :is-warning="!isCanSupply"
      />

      <div class="collateral mt-3">
        <div class="collateral-label">Collateral Only</div>

        <j-toggle
          v-model="collateralOnly"
          size="small"
          :disabled="!isCanSupply"
        />
      </div>

      <div class="info-card mt-3 info-supply">
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

      <div class="supply-card__action mt-3">
        <market-dialog-action-btn
          variant="blue"
          :loading="isLoading"
          :pool="selectedPool?.raw.pool"
          :disabled="!isCanSupply || amount >= balance"
          pill
          @click-handler="supply"
        >
          <i-metrics-complete class="complete-icon" /> Supply {{ selectedPool?.asset.symbol }}
        </market-dialog-action-btn>
      </div>
    </div>
  </div>
</template>

<style lang="scss">
.supply-card {
  max-width: 400px;
  width: 100%;
  height: fit-content;
  background-color: color-mix(in oklab, $card 50%, transparent);
  padding: 20px;
  border: 1px solid $border-color;
  border-radius: 14px;

  .info-card {
    background-color: color-mix(in oklab, $new-secondary 30%, transparent);
    border: 1px solid $border-color;
    border-radius: 14px;
    transition: border-color 0.2s ease;
    padding: 16px;
  }

  .input-wrapper {
    display: flex;
    flex-direction: column;

    .wallet-balance {
      display: flex;
      align-items: center;
      justify-content: space-between;
      font-size: 12px;
      color: $muted-foreground;
      margin-bottom: 8px;

      &__value {
        font-family: $font-JetBrainsMono;
      }
    }

    .input-block {
      padding: 0;

      &.active {
        background-color: rgba(0, 211, 238, 0.03);
        border-color: rgba(0, 211, 238, 0.3);
      }

      &.error {
        background-color: rgb(244 63 94 / 10%);
        border-color: #f43f5e;
      }

      &__top {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 16px;

        .input-group {
          border: none !important;
        }

        .input-wrapper {
          height: 100%;
        }

        input {
          height: 100%;
          text-align: right;
          font-family: $font-JetBrainsMono;
          font-weight: 500;
          font-size: 1.4rem;
          color: $foreground;

          &::placeholder {
            color: $muted-foreground;
            opacity: 0.5;
          }
        }
      }

      &__btns {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 16px 12px;
      }

      .select-amount {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 6px;
        font-size: 12px;
        color: $muted-foreground;

        span {
          padding: 4px 10px;
          font-size: 11px;
          text-transform: uppercase;
          border-radius: 6px;
          color: $muted-foreground;
          background-color: color-mix(in oklab, $new-secondary 60%, transparent);
          transition: all 0.1s ease;
          cursor: pointer;

          &:hover {
            color: $foreground;
          }

          &.active {
            color: $supply;
            background-color: rgba(0, 211, 238, 0.15);
          }
        }
      }

      .amount-to-dollar {
        font-size: 12px;
        font-family: $font-JetBrainsMono;
        color: $muted-foreground;
      }
    }

    .input-errors {
      color: #f43f5e;
      margin: 8px 0 12px;
      font-size: 12px;
    }
  }

  .collateral {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 14px;
    color: $muted-foreground;
  }

  .info-supply {
    background-color: rgba(0, 211, 238, 0.04);
    border-color: rgba(0, 211, 238, 0.1);

    &__header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      margin-bottom: 12px;

      .info-title {
        font-size: 12px;
        color: $muted-foreground;
        display: flex;
        align-items: center;
        gap: 8px;

        &::before {
          content: '';
          width: 8px;
          height: 8px;
          border-radius: 50%;
          background-color: #22d3ee;
          display: block;
        }
      }

      .info-apy {
        color: #22d3ee;
        font-family: $font-JetBrainsMono;
        font-weight: 700;
        font-size: 20px;
      }
    }

    &__body {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 16px;

      .info-detail {
        width: 100%;
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        overflow: hidden;

        &__title {
          font-size: 10px;
          text-transform: uppercase;
          color: $muted-foreground;
          opacity: 0.7;
        }

        &__value {
          width: 100%;
          font-family: $font-JetBrainsMono;
          font-size: 14px;
          font-weight: 600;
          color: $foreground;
          overflow: hidden;
          text-overflow: ellipsis;
        }
      }
    }
  }

  .info-summary {
    padding: 0;

    &__header {
      font-size: 11px;
      text-transform: uppercase;
      color: $muted-foreground;
      padding: 10px 16px;
      border-bottom: 1px solid $border-color;
    }

    .summary-list {
      padding: 16px;
      display: flex;
      flex-direction: column;
      gap: 12px;

      &__item {
        height: 16px;
        display: flex;
        align-items: center;
        justify-content: space-between;
        font-size: 12px;
        color: $foreground;

        .label {
          color: $muted-foreground;
        }

        .value {
          font-family: $font-JetBrainsMono;
          opacity: 0.8;
        }
      }
    }
  }
}
</style>
