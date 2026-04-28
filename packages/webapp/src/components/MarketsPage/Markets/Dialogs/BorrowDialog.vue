<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'

const props = defineProps<{ data?: MarketTableItem }>()

const poolData = toRef(props, 'data')

const dialog = defineModel({ default: false })

provide('selectedPool', poolData)
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="dialog-default"
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

    <div class="dialog-default__body">
      <borrow-window
        :with-selected-pool="false"
        opened
      />
    </div>
  </j-dialog>
</template>
