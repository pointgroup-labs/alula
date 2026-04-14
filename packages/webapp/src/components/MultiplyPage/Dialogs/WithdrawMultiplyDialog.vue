<script lang="ts" setup>
import type { MultiplyAccountTableItem, MultiplyTableItem, MultiplyVaultItem } from '~/types/table'

const {
  data,
} = defineProps<{
  data?: MultiplyTableItem | MultiplyAccountTableItem | MultiplyVaultItem
}>()

const dialog = defineModel({ default: false })
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="pool-dialog dialog-default"
  >
    <template #header>
      <div class="dialog-default__title">
        <img
          :src="data?.asset.icon"
          :alt="`${data?.asset.symbol} icon`"
        >
        <span>Withdraw {{ data?.asset.symbol }}</span>
      </div>
    </template>

    <div class="dialog-default__body">
      <withdraw-multiply-window
        :data="data"
        :opened="dialog"
      />
    </div>
  </j-dialog>
</template>
