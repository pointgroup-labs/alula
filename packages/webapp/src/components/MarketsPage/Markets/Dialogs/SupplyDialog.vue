<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'

const props = defineProps<{ data?: MarketTableItem }>()

const dialog = defineModel({ default: false })
const isOpen = ref(false)

const poolData = toRef(props, 'data')

watch(dialog, (v) => {
  setTimeout(() => isOpen.value = v, v ? 0 : 500)
})

provide('selectedPool', poolData)
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
      <supply-window
        :with-selected-pool="false"
        opened
      />
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
      font-family: $font-family-base;
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
