<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'
import { CLEAR_DIALOG_TIMEOUT, POOL_REMAINING_BALANCE } from '~/config'
import { truncatePercent } from '~/utils'

const props = defineProps<{ data?: MarketTableItem }>()

const poolData = toRef(props, 'data')

const dialog = defineModel({ default: false })
const isOpen = ref(false)

const {
  agree,
  isLoading,
  isLoadingFee,
  amount,
  healthFactor,
  availableToBorrow,
  isCanBorrow,
  attentionText,
  infoPanelData,
  borrow,
  stopBorrowWatchers,
} = useBorrowDialog(poolData, dialog)

watch(dialog, (v) => {
  setTimeout(() => isOpen.value = v, v ? 0 : 500)
  if (!v) {
    stopBorrowWatchers()
    setTimeout(() => { amount.value = 0 }, CLEAR_DIALOG_TIMEOUT)
    agree.value = false
  }
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

    <div
      v-if="isOpen"
      class="dialog-default__body"
    >
      <input-widget
        v-model="amount"
        class="borrow-input mb-2"
        :balance="availableToBorrow"
        :fee="POOL_REMAINING_BALANCE"
        label-left="Available"
        :label-right="`${formatPrice(availableToBorrow ?? 0, 0, 4)} ${data?.asset.symbol}`"
        :price="poolData?.price"
        :reset="dialog"
        variant="purple"
        :rules="[
          (v: any) => {
            return Number(v) < availableToBorrow * 1.1 || 'Borrow limit exceeded'
          },
        ]"
      />

      <template v-if="data">
        <!-- Pool info -->
        <info-panel
          :data="infoPanelData.poolInfo!.data"
          variant="purple"
        />

        <!-- Health -->
        <info-panel
          :data="infoPanelData.health!.data"
          variant="purple"
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
          :data="infoPanelData.fees!.data"
          variant="purple"
        >
          <template #txFee="{ item }">
            <j-loading-spinner
              v-if="isLoadingFee"
              width="14px"
              style="padding: 0; width: 20px; height: 20px; margin: 0 auto;"
            />
            <template v-else>
              {{ item.value }}
            </template>
          </template>
        </info-panel>
      </template>

      <warning-block
        :text="attentionText"
        :is-warning="!isCanBorrow"
      />

      <div class="extra-info">
        <j-checkbox
          v-model="agree"
          :disabled="!isCanBorrow"
          color="#8a8df4"
        >
          <div class="extra-info__label">
            I acknowledge the risks involved.
          </div>
        </j-checkbox>
      </div>

      <div class="extra-info">
        <div class="extra-info__label">Borrow APY</div>
        <div class="extra-info__value text-num">{{ data?.borrow_apy }}</div>
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
