<script lang="ts" setup>
import type { MultiplyTableItem, MultiplyVaultItem } from '~/types/table'

const {
  data,
} = defineProps<{
  data?: MultiplyTableItem | MultiplyVaultItem
}>()

const dialog = defineModel({ default: false })
const headerOptionsRef = ref()
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="multiply-dialog dialog-default"
  >
    <template #header>
      <div class="multiply-dialog__title">
        <span>Close</span>
      </div>

      <div
        id="multiply-header-options"
        ref="headerOptionsRef"
      />
    </template>

    <withdraw-multiply-window
      :vault="data"
      :opened="dialog"
      :teleport-target="headerOptionsRef"
    />
  </j-dialog>
</template>
