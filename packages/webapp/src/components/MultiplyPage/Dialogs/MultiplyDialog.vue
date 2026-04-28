<script lang="ts" setup>
import type { MultiplyVaultItem } from '~/types/table'

const {
  data,
} = defineProps<{
  data?: MultiplyVaultItem
  hasPosition?: boolean
}>()

const dialog = defineModel<boolean>({ default: false })
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="dialog-default"
  >
    <template #header>
      <div class="dialog-default__title">
        <template v-if="hasPosition">
          Add To Position
        </template>
        <template v-else>
          Open Position
        </template>
      </div>
    </template>

    <div class="dialog-default__body">
      <multiply-window
        v-if="data"
        :vault="data"
        compact
      />
    </div>
  </j-dialog>
</template>
