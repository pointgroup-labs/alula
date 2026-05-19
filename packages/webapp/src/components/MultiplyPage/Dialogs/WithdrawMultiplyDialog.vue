<script lang="ts" setup>
import type { MultiplyTableItem, MultiplyVaultItem } from '~/types/table'

const {
  data,
} = defineProps<{
  data?: MultiplyTableItem | MultiplyVaultItem
}>()

const dialog = defineModel({ default: false })
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="dialog-default"
  >
    <template #header>
      <div class="dialog-default__title">
        <span>Close Position</span>
      </div>
    </template>

    <div class="dialog-default__body">
      <withdraw-multiply-window
        :vault="data"
        :opened="dialog"
      />
    </div>
  </j-dialog>
</template>
