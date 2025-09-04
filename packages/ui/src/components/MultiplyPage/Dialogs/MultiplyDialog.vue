<script lang="ts" setup>
import type { MultiplyTableItem } from '~/types/table'
import { CLEAR_DIALOG_TIMEOUT, RELOAD_FEE_INTERVAL } from '~/config'
import { bigintToNumber, destructurePoolAsset, focusInput, formatPrice, shortenAddress, truncatePercent } from '~/utils'

const {
  data,
} = defineProps<{
  data?: MultiplyTableItem
}>()

const { generateExplorerLink } = useExplorerLink()

function calcRemainingMultiplyUSD(
  borrowAvailableInUsd: number,
  poolPrice: number,
  selectedMultiplier: number,
): number {
  if (selectedMultiplier <= 1) {
    return borrowAvailableInUsd
  }
  return borrowAvailableInUsd / poolPrice / selectedMultiplier
}

const marketsStore = useMarketsStore()
const market = useMarket()

const amount = toRef(market, 'depositAmount')

const clientStore = useClientStore()
const jLendClient = computed(() => clientStore.jLendClient)

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const isDepositMultiply = ref(true)

const multiplyAssets = computed(() => {
  const depositAsset = data?.asset
  const borrowAsset = data?.borrowAsset
  return [depositAsset, borrowAsset]
})

const depositAsset = computed(() => multiplyAssets.value[isDepositMultiply.value ? 0 : 1])
const borrowAsset = computed(() => multiplyAssets.value[isDepositMultiply.value ? 1 : 0])

const balance = computed(() => {
  if (!data) {
    return 0
  }
  const poolAsset = isDepositMultiply.value ? data.depositPool.name : data.borrowPool.name
  if (poolAsset === 'native') {
    return wallet.nativeBalance
  }
  const [, asset_issuer] = destructurePoolAsset(poolAsset)
  return wallet.getAssetBalance(String(asset_issuer))
})

const precentFromMaxMultiply = ref(90)

const maxMultiply = computed(() => data?.multiplier || 0)
const selectedMultiplier = computed(() => {
  return Number((precentFromMaxMultiply.value / 100) * maxMultiply.value).toFixed(2)
})

const loading = computed(() => marketsStore.poolActiveAddress === data?.depositPool.pool_address)
const reloadFee = ref(false)

const txFee = ref(0)

watchDebounced([
  () => data,
  reloadFee,
  publicKey,
], async ([d, _r]) => {
  if (!d || !publicKey.value) {
    return
  }
  const tx = await jLendClient.value?.sdk.leverageTx(
    publicKey.value,
    d?.depositPool.pool_address || '',
    d?.borrowPool.pool_address || '',
    isDepositMultiply.value,
    1,
    2,
  )
  txFee.value = jLendClient.value.sdk.getTransactionFee(tx)
}, { immediate: true, debounce: 300 })

const supplyLimit = ref(0)

const infoTableData = computed(() => {
  if (!data) {
    return []
  }

  const depositPoolData = isDepositMultiply.value ? data.depositPool : data.borrowPool

  const borrowPoolData = data.borrowPool
  const borrowAvailable = bigintToNumber(borrowPoolData.available, clientStore.assetDecimals)
  const borrowAvailableInUsd = Number(borrowAvailable) * Number(borrowPoolData.pool_price)

  const maxMultiplyTicker = isDepositMultiply.value ? data.depositPool.token_ticker : data.borrowPool.token_ticker

  // eslint-disable-next-line vue/no-side-effects-in-computed-properties
  supplyLimit.value
   = calcRemainingMultiplyUSD(borrowAvailableInUsd, Number(depositPoolData?.pool_price || 0), Number(selectedMultiplier.value) || 0)

  return [
    {
      name: 'liquidity',
      label: 'Liquidity Available',
      value: `${formatPrice(borrowAvailable || 0, 2, 2)} ${borrowPoolData.token_ticker}`,
    },
    {
      name: 'maxApy',
      label: 'Max APY',
      value: `${truncatePercent(data.maxAPY || 0, 2)} %`,
    },
    {
      name: 'maxMultiply',
      label: 'Max Multiply',
      value: `${formatPrice(Number(supplyLimit.value || 0).toFixed(2), 2)} ${maxMultiplyTicker}`,
    },
    {
      name: 'multiplier',
      label: 'Avg. Multiplier',
      value: '90.00 %',
    },
    {
      name: 'supplied',
      label: 'Total Supplied',
      value: `${formatPrice(data.supplied || 0, 2, 2)} ${data.asset.symbol}`,
    },
    {
      name: 'txFee',
      label: 'Transaction Fee',
      value: `${txFee.value} XLM`,
    },
  ]
})

const dialog = defineModel<boolean>({ default: false })

function swapAsset() {
  isDepositMultiply.value = !isDepositMultiply.value
}

async function leverage() {
  if (!publicKey.value || !data?.depositPool.pool_address) {
    return
  }
  if (!amount.value || amount.value <= 0) {
    focusInput('.multiply-dialog')
    return
  }
  const deposit_pool_address = data?.depositPool.pool_address
  const borrow_pool_address = marketsStore.state.pools.find(p => p.token_ticker === 'XLM')?.pool_address || ''
  const asset_code = data?.depositPool.token_ticker
  if (!deposit_pool_address || !borrow_pool_address) {
    return
  }

  await market.leverage(
    deposit_pool_address,
    borrow_pool_address,
    isDepositMultiply.value,
    amount.value,
    Number(selectedMultiplier.value),
    asset_code,
  )
}

let interval: string | number | NodeJS.Timeout | undefined

watch(dialog, async (v) => {
  clearInterval(interval)
  if (!v) {
    setTimeout(() => {
      amount.value = 0
    }, CLEAR_DIALOG_TIMEOUT)
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
    class-name="multiply-dialog"
  >
    <template #header>
      <div class="multiply-dialog__title">
        <span>Multiply</span>
      </div>

    </template>

    <div class="multiply-dialog__body">
      <div class="multiply-dialog__data with-border">

        <input-widget
          v-model="amount"
          :balance="balance"
          :limit="supplyLimit"
          class="multiply-dialog__input"
          label-left="You Deposit"
          :rules="[
            (v) => {
              return v && Number(v) < balance || 'Insufficient balance'
            },
            (v) => {
              return (supplyLimit <= 0 || Number(v) <= supplyLimit) || 'Pool leverage limit'
            },
          ]"
        >
          <template #label-right>
            Wallet: {{ balance }} {{ depositAsset?.name }}
          </template>
          <template #prepend>
            <j-popover
              position="bottom"
              :teleport-to-body="false"
              close-popup
            >
              <div
                class="popover-borrow-asset"
                @click="swapAsset"
              >
                <img
                  :src="borrowAsset?.icon"
                  :alt="`${borrowAsset?.name} icon`"
                >
                {{ borrowAsset?.name }}
              </div>
              <template #target="{ active }">
                <j-tooltip>
                  <img
                    :src="depositAsset?.icon"
                    :alt="`${depositAsset?.name} icon`"
                  >
                  <i-app-arrow-up
                    class="arrow-icon"
                    :class="{ 'arrow-icon--active': active }"
                  />
                  <template #content>
                    Change multiply asset to {{ borrowAsset?.name }}
                  </template>
                </j-tooltip>
              </template>
            </j-popover>
          </template>
        </input-widget>

        <div
          v-if="infoTableData.length > 0"
          class="dialog-info-table"
        >
          <div
            v-for="item in infoTableData"
            :key="item?.label"
            class="dialog-info-table__item"
          >
            <span>{{ item?.label }}</span>
            <template v-if="item?.name === 'contract'">
              <a
                :href="generateExplorerLink(String(item?.value), 'contract')"
                target="_blank"
              >{{
                 shortenAddress(item?.value, 5) }}
                <i-app-export-icon />
              </a>
            </template>
            <span v-else>{{ item?.value }}</span>
          </div>
        </div>

        <multiply-select
          v-model="precentFromMaxMultiply"
          :multiplier="selectedMultiplier"
          :max-multiply="maxMultiply"
        />

        <div class="multiply-dialog-action">
          <market-dialog-action-btn
            variant="primary"
            :loading="loading"
            :pool="data?.depositPool"
            :disabled="Number(selectedMultiplier) < 1"
            @click-handler="leverage"
          >
            Multiply {{ data?.asset.symbol }}
          </market-dialog-action-btn>
        </div>
      </div>

      <multiply-apy-chart
        :data="data"
        :is-deposit-multiply="isDepositMultiply"
      />
    </div>
  </j-dialog>
</template>

<style lang="scss">
.multiply-dialog {
  .modal-dialog {
    width: min-content;

    @media (max-width: $breakpoint-xs) {
      width: 100%;
    }
  }

  .j-input__prepend {
    width: 40px;
    min-width: 40px;

    .popover {
      &-body {
        padding: $spacing-12;
      }

      .popover-borrow-asset {
        display: flex;
        align-items: center;
        gap: $spacing-6;
        cursor: pointer;
      }
    }

    .popover-target {
      & > div {
        display: flex;
        align-items: center;
        gap: 2px;
        cursor: pointer;
      }
    }

    .arrow-icon {
      width: 18px;
      height: 18px;
      transform: rotate(180deg);

      &--active {
        transform: rotate(0deg);
      }
    }

    img {
      width: 32px;
      height: 32px;
      object-fit: contain;
      border-radius: 50%;
    }
  }

  &__title {
    color: $dark;
    font-size: 20px;
    font-style: normal;
    font-weight: 500;
    line-height: 20px;
  }

  &__body {
    padding-top: $spacing-16;
    display: flex;
    flex-direction: row;
    gap: 48px;

    @media (max-width: $breakpoint-xs) {
      flex-direction: column-reverse;
      gap: $spacing-16;
    }
  }

  &__data {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: $spacing-16;

    @media (max-width: $breakpoint-xs) {
      min-width: 100%;
      width: 100%;
    }

    &.with-border {
      &::after {
        content: '';
        width: 1px;
        height: 100%;
        background-color: $neutral-5;
        position: absolute;
        top: 0;
        right: -24px;

        @media (max-width: $breakpoint-xs) {
          display: none;
        }
      }
    }
  }

  .multiply-dialog-action {
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
  .multiply-dialog {
    .j-input__prepend .popover {
      .popover-borrow-asset {
        color: #fff;
      }
    }

    &__data {
      &.with-border {
        &::after {
          background: $neutral-18;
        }
      }
    }
  }
}
</style>
