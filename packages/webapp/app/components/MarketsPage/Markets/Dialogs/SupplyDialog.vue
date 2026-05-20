<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'

const props = defineProps<{ data?: MarketTableItem }>()

const dialog = defineModel({ default: false })

const poolData = toRef(props, 'data')

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
          :src="poolData?.asset.icon"
          :alt="`${poolData?.asset.symbol} icon`"
        >
        <span>Supply {{ poolData?.asset.symbol }}</span>
      </div>
    </template>

    <div class="dialog-default__body">
      <supply-window
        :with-selected-pool="false"
        opened
      />
    </div>
  </j-dialog>
</template>
