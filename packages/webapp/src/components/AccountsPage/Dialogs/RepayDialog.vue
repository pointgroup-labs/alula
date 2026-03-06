<script lang="ts" setup>
import { CLEAR_DIALOG_TIMEOUT } from '~/config'

const dialog = defineModel({ default: false })
const isOpen = ref(false)

const isValidate = ref(true)

const {
  poolData,
  asset,
  price,
  debt,
  balance,
  infoPanelData,
  isLoadingFee,
  amount,
  loading: isLoading,
  repay: doRepay,
  stopRepayWatcher,
} = useRepayDialog(dialog)

async function repay() {
  isValidate.value = false
  await doRepay()
  isValidate.value = true
}

watch(poolData, (r) => {
  if (!r) {
    dialog.value = false
  }
})

watch(dialog, (v) => {
  setTimeout(() => isOpen.value = v, v ? 0 : 500)
  if (!v) {
    stopRepayWatcher()
    setTimeout(() => { amount.value = 0 }, CLEAR_DIALOG_TIMEOUT)
  }
})
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="dialog-default"
  >
    <template #header>
      <div class="dialog-default__title">
        <img
          :src="asset.icon"
          :alt="`${asset.symbol} icon`"
        >
        <span>Repay {{ asset.symbol }}</span>

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
        class="repay-dialog__input mb-2"
        :balance="balance"
        :limit="debt"
        label-left="Balance"
        variant="purple"
        :label-right="`${formatPrice(balance ?? 0, 0, 4)} ${asset.symbol}`"
        :reset="dialog"
        :price="Number(price)"
        :rules="[
          (v) => {
            return !isValidate || Number(v) <= balance || 'Insufficient balance'
          },
        ]"
      />

      <template v-if="Object.keys(infoPanelData).length > 0">
        <!-- Balances -->
        <info-panel
          :data="infoPanelData.balances!.data"
          variant="purple"
        />
        <!-- Fee -->
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

      <div class="dialog-default__action mt-2">
        <j-btn
          :loading="isLoading"
          variant="purple"
          size="lg"
          pill
          @click="repay"
        >
          Repay {{ asset.symbol }}
        </j-btn>
      </div>
    </div>
  </j-dialog>
</template>
