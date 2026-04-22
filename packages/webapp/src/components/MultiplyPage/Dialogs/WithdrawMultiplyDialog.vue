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
    class-name="multiply-dialog dialog-default"
  >
    <template #header>
      <div class="multiply-dialog__title">
        <span>Close Position</span>
      </div>
    </template>

    <withdraw-multiply-window
      :vault="data"
      :opened="dialog"
    />
  </j-dialog>
</template>
