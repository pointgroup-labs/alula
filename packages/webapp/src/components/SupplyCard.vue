<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { calcFee } from '@alula/client-sdk/src/utils'
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
        />
      </div>

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

      <warning-block
        v-if="!isCanSupply"
        :text="attentionText"
        :is-warning="!isCanSupply"
        class="mt-3"
      />

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
