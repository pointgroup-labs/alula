<script lang="ts" setup>
import { CLEAR_DIALOG_TIMEOUT } from '~/config'

const dialog = defineModel({ default: false })
const isOpen = ref(false)

const isValidate = ref(true)

const {
  poolData,
  asset,
  price,
  collateralBalance,
  availableToWithdrawWithPoolLimit,
  infoPanelData,
  isLoadingFee,
  amount,
  collateralOnly,
  loading: isLoading,
  withdraw: doWithdraw,
  stopWithdrawWatchers,
} = useWithdrawDialog(dialog)

async function withdraw() {
  isValidate.value = false
  await doWithdraw()
  isValidate.value = true
}

watch(poolData, (r) => {
  if (!r) {
    dialog.value = false
  }
})

watch(dialog, (v) => {
  setTimeout(() => {
    isOpen.value = dialog.value
  }, dialog.value ? 0 : 500)
  if (!v) {
    setTimeout(() => { amount.value = 0 }, CLEAR_DIALOG_TIMEOUT)
    stopWithdrawWatchers()
    collateralOnly.value = false
  }
})
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name=" dialog-default"
  >
    <template #header>
      <div class="dialog-default__title">
        <img
          :src="asset.icon"
          :alt="`${asset.symbol} icon`"
        >
        <span>Withdraw {{ asset.symbol }}</span>

        <reload-coundown
          v-if="dialog"
          color="#e8edf5"
        />
      </div>
    </template>

    <div
      v-if="isOpen"
      class="dialog-default__body"
    >
      <input-widget
        v-model="amount"
        :balance="availableToWithdrawWithPoolLimit"
        class="withdraw-dialog__input mb-2"
        :price="Number(price)"
        label-left="Amount"
        :label-right="`${formatPrice(availableToWithdrawWithPoolLimit ?? 0, 0, 4)} ${asset.symbol}`"
        :reset="dialog"
        variant="cyan"
        :rules="[
          (v) => {
            return !isValidate || Number(v) <= availableToWithdrawWithPoolLimit || 'Withdraw limit exceeded'
          },
        ]"
      />

      <template v-if="Object.keys(infoPanelData).length > 0">
        <!-- Balances -->
        <info-panel
          :data="infoPanelData.balances!.data"
        />

        <!-- Health -->
        <info-panel
          :data="infoPanelData.health!.data"
        />

        <!-- Pool Info -->
        <info-panel
          :data="infoPanelData.poolInfo!.data"
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

        <!-- Fees -->
        <info-panel
          :data="infoPanelData.fees!.data"
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

      <j-toggle
        v-if="collateralBalance > 0"
        v-model="collateralOnly"
        color="#00c950"
        class="my-2"
      >
        <template #append>
          Collateral Balance
        </template>
      </j-toggle>

      <div class="dialog-default__action mt-2">
        <j-btn
          :loading="isLoading"
          variant="cyan"
          size="lg"
          pill
          @click="withdraw"
        >
          Withdraw {{ asset.symbol }}
        </j-btn>
      </div>
    </div>
  </j-dialog>
</template>
