<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { calcFee } from '@alula/client-sdk/src/utils'
import Decimal from 'decimal.js'
import { POOL_REMAINING_BALANCE } from '~/config'
import { /* focusInput, */ formatPrice } from '~/utils'

const selectedPool = inject<Ref<MarketTableItem>>('selectedPool')

// const dialog = defineModel({ default: false })

const route = useRoute()
const router = useRouter()

const { generateExplorerLink } = useExplorerLink()

const marketsStore = useMarketsStore()
const market = useMarketActions()

const amount = toRef(market, 'depositAmount')

// const wallet = useWallet()
// const publicKey = computed(() => wallet.publicKey)

const {
//   marketClient,
  collateralOnly,
  balance,
  txFee,
//   reloadFee,
  isLoadingFee,
  supplyLimit,
  limitLabel,
  contractAddress,
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

// async function supply() {
//   try {
//     if (!publicKey.value || !selectedPool?.value?.raw.pool.pool_address) {
//       return
//     }
//     if (!amount.value || amount.value <= 0) {
//       focusInput('.supply-dialog__input')
//       return
//     }
//     marketsStore.poolActiveAddress = selectedPool?.value?.raw.pool.pool_address

//     const marketProps = {
//       market: marketsStore.selectedMarketName,
//       client: marketClient.value!,
//       pool_address: selectedPool?.value?.raw.pool.pool_address,
//       amount: amount.value,
//       asset_data: selectedPool?.value?.raw.pool.name,
//     }
//     collateralOnly.value
//       ? await market.addCollateral(marketProps)
//       : await market.deposit(marketProps)

//     marketsStore.dialogSupply = false
//   } finally {
//     marketsStore.poolActiveAddress = undefined
//   }
// }

// let interval: string | number | NodeJS.Timeout | undefined

// watch(dialog, async (v) => {
//   clearInterval(interval)
//   if (!v) {
//     setTimeout(() => {
//       amount.value = 0
//     }, CLEAR_DIALOG_TIMEOUT)
//     collateralOnly.value = false
//     return
//   }

//   interval = setInterval(() => {
//     reloadFee.value = true
//     nextTick(() => {
//       reloadFee.value = false
//     })
//   }, RELOAD_FEE_INTERVAL)
// })

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
  <div
    class="supply-card"
  >

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

  

      <div
        v-if="amount > 0 && selectedPool"
        class="dialog-info-table"
      >
        <!-- Supply Limit -->
        <div
          class="dialog-info-table__item"
        >
          <span>Supply Limit</span>
          <span>{{ limitLabel }} {{ limitLabel !== '-' ? selectedPool?.asset.symbol : '' }}</span>
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

        <!-- Open LTV  -->
        <div
          class="dialog-info-table__item"
        >
          <span>Open LTV </span>
          <span>{{ selectedPool?.open_ltv }}</span>
        </div>

        <!-- Util Rate -->
        <div
          class="dialog-info-table__item"
        >
          <span>Utilization Rate</span>
          <span>{{ selectedPool?.utilization_rate }}</span>
        </div>

        <!-- Market Fee -->
        <div
          class="dialog-info-table__item"
        >
          <span>Operation Fee</span>

          <span>{{ formatPrice(marketFee) }} XLM</span>
        </div>

        <!-- Transaction Fee -->
        <div
          class="dialog-info-table__item"
        >
          <span>Transaction Fee</span>
          <j-loading-spinner
            v-if="isLoadingFee"
            width="14px"
            style="margin:0 20px 0 auto;"
          />
          <span v-else>{{ txFee }} XLM</span>
        </div>

        <div class="separator" />
      </div>

      <warning-block
        v-if="!isCanSupply"
        :text="attentionText"
        :is-warning="!isCanSupply"
      />

      <div class="collateral">
        <div class="collateral-label">Collateral Only</div>

        <j-toggle
          v-model="collateralOnly"
          size="small"
          :disabled="!isCanSupply"
        />
      </div>

          <div class="info-card">
        wef
      </div>

      <!-- <div class="extra-info">
        <div class="extra-info__label">Supply APY</div>
        <div class="extra-info__value">{{ selectedPool?.deposit_apy }}</div>
      </div> -->

      <div class="supply-card__action">
        <!-- <market-dialog-action-btn
          variant="blue"
          :loading="isLoading"
          :pool="poolData?.raw.pool"
          :disabled="!isCanSupply || amount >= balance"
          @click-handler="supply"
        >
          Supply {{ poolData?.asset.symbol }}
        </market-dialog-action-btn> -->
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
    margin-top: 12px;
    font-size: 14px;
    color: $muted-foreground;
  }
}
</style>
