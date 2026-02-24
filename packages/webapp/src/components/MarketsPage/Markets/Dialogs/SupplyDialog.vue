<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { calcFee } from '@alula/client-sdk/src/utils'
import { CLEAR_DIALOG_TIMEOUT, POOL_REMAINING_BALANCE, RELOAD_FEE_INTERVAL } from '~/config'
import { focusInput, formatPrice } from '~/utils'

const props = defineProps<{ data?: MarketTableItem }>()

const dialog = defineModel({ default: false })

const { generateExplorerLink } = useExplorerLink()

const marketsStore = useMarketsStore()
const market = useMarketActions()

const poolData = toRef(props, 'data')

const amount = toRef(market, 'depositAmount')

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const {
  marketClient,
  collateralOnly,
  balance,
  txFee,
  reloadFee,
  isLoadingFee,
  supplyLimit,
  limitLabel,
  contractAddress,
  isLoading,
  isCanSupply,
  attentionText,
} = useSupplyDialog(poolData)

const marketFee = computed(() => {
  const marketFeeBps = collateralOnly.value
    ? poolData.value?.raw.pool.config.fee_config.add_collateral_fee_bps
    : poolData.value?.raw.pool.config.fee_config.deposit_fee_bps
  return calcFee(Number(amount.value || 0), marketFeeBps || 0)
})

const reserveAmount = computed(() => poolData.value?.raw.pool.token_symbol === 'native' ? 2 : 0)

async function supply() {
  try {
    if (!publicKey.value || !poolData.value?.raw.pool.pool_address) {
      return
    }
    if (!amount.value || amount.value <= 0) {
      focusInput('.supply-dialog__input')
      return
    }
    marketsStore.poolActiveAddress = poolData.value?.raw.pool.pool_address

    const marketProps = {
      market: marketsStore.selectedMarketName,
      client: marketClient.value!,
      pool_address: poolData.value?.raw.pool.pool_address,
      amount: amount.value,
      asset_data: poolData.value?.raw.pool.name,
    }
    collateralOnly.value
      ? await market.addCollateral(marketProps)
      : await market.deposit(marketProps)

    marketsStore.dialogSupply = false
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
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="supply-dialog dialog-default"
  >
    <template #header>
      <div class="dialog-default__title">
        <img
          :src="poolData?.asset.icon"
          :alt="`${poolData?.asset.symbol} icon`"
        >
        <span>Supply {{ poolData?.asset.symbol }}</span>
      </div>

      <div class="dialog-balance">
        <div class="dialog-balance__label">Balance:</div>
        <div class="dialog-balance__value">{{ shortenNumber(balance) }} {{ poolData?.asset.symbol }}</div>
      </div>
    </template>

    <div class="dialog-default__body">
      <input-widget
        v-model="amount"
        :balance="balance"
        :limit="Number(supplyLimit) || 0"
        :fee="POOL_REMAINING_BALANCE + txFee + reserveAmount"
        class="dialog-default__input"
        :rules="[
          (v) => {
            return v && Number(v) < balance || 'Insufficient balance'
          },
          (v) => {
            return (supplyLimit <= 0 || Number(v) <= supplyLimit) || 'Pool supply limit'
          },
        ]"
      />

      <div
        v-if="poolData"
        class="dialog-info-table"
      >
        <!-- Supply Limit -->
        <div
          class="dialog-info-table__item"
        >
          <span>Supply Limit</span>
          <span>{{ limitLabel }} {{ limitLabel !== '-' ? poolData?.asset.symbol : '' }}</span>
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
          <span>{{ poolData.open_ltv }}</span>
        </div>

        <!-- Util Rate -->
        <div
          class="dialog-info-table__item"
        >
          <span>Utilization Rate</span>
          <span>{{ poolData.utilization_rate }}</span>
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

      <div class="extra-info">
        <div class="extra-info__label">Collateral Only</div>

        <j-toggle
          v-model="collateralOnly"
          size="small"
          :disabled="!isCanSupply"
        />
      </div>

      <div class="extra-info">
        <div class="extra-info__label">Supply APY</div>
        <div class="extra-info__value">{{ poolData?.deposit_apy }}</div>
      </div>

      <div class="dialog-default__action">
        <market-dialog-action-btn
          variant="blue"
          :loading="isLoading"
          :pool="poolData?.raw.pool"
          :disabled="!isCanSupply || amount >= balance"
          @click-handler="supply"
        >
          Supply {{ poolData?.asset.symbol }}
        </market-dialog-action-btn>
      </div>
    </div>
  </j-dialog>
</template>

<style lang="scss">
.supply-dialog {
  .modal-content {
    max-width: 442px;
  }

  .extra-info {
    display: flex;
    align-items: center;
    justify-content: space-between;

    &__label {
      font-size: 14px;
      font-style: normal;
      font-weight: 500;
      line-height: 16px;
    }

    &__value {
      font-family: $font-Inter;
      font-size: 24px;
      font-style: normal;
      font-weight: 700;
      line-height: 36px;
    }
  }

  .j-input__label {
    display: none;
  }
}
</style>
