<script lang="ts" setup>
import type { MultiplyVaultItem } from '~/types/table'

const {
  data,
} = defineProps<{
  data?: MultiplyVaultItem
}>()

const dialog = defineModel<boolean>({ default: false })
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="multiply-dialog"
  >
    <template #header>
      <div class="multiply-dialog__title">
        Open Multiply
      </div>
    </template>

    <multiply-window
      v-if="data"
      :vault="data"
      compact
    />
  </j-dialog>
</template>

<style lang="scss">
.multiply-dialog {
  .modal-dialog {
    width: 100%;
    max-width: 500px;
  }

  .modal-content {
    border-radius: 28px;
    background: linear-gradient(180deg, rgba(17, 24, 39, 0.98) 0%, rgba(13, 18, 31, 0.98) 100%);
    border: 1px solid $border-primary;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.35);
  }

  .modal-header {
    padding: 24px 24px 16px;
    border-bottom: 1px solid $surface-neutral-08;
    background: transparent;
  }

  .modal-body {
    padding: 24px;
    background: transparent;
  }

  &__title {
    font-size: 22px;
    font-weight: 700;
    color: $text-primary;
  }
}
</style>
