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
      <div class="supply-dialog__title">
        <img
          :src="poolData?.asset.icon"
          :alt="`${poolData?.asset.symbol} icon`"
        >
        <span>Supply {{ poolData?.asset.symbol }}</span>
      </div>
    </template>

    <div class="supply-dialog__body">
      <input-widget
        v-model="amount"
        :balance="balance"
        :limit="Number(supplyLimit) || 0"
        :fee="POOL_REMAINING_BALANCE + txFee + reserveAmount"
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
          Balance: {{ balance }} {{ poolData?.asset.symbol }}
        </template>
      </input-widget>

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
      </div>

      <warning-block
        v-if="!isCanSupply"
        :text="attentionText"
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
          <span>{{ poolData?.deposit_apy }}</span>
        </div>

        <market-dialog-action-btn
          variant="primary"
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
