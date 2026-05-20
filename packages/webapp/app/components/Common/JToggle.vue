<script lang="ts" setup>
const {
  disabled = false,
} = defineProps<{
  disabled?: boolean
  color?: string
}>()

const slots = defineSlots()

const value = defineModel({ default: false })

function toggleHandler() {
  if (disabled) {
    return
  }
  value.value = !value.value
}
</script>

<template>
  <div
    class="j-toggle"
    :style="{ '--toggle-color': color }"
    @click="toggleHandler"
  >
    <div
      v-if="slots?.prepend"
      class="j-toggle__label"
    >
      <slot name="prepend" />
    </div>
    <b-form-checkbox
      v-model="value"
      :disabled="disabled"
      switch
    />
    <div
      v-if="slots?.append"
      class="j-toggle__label"
    >
      <slot name="append" />
    </div>
  </div>
</template>

<style lang="scss">
.j-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;

  &:has(.form-check-input:disabled) {
    cursor: not-allowed;
  }

  &__label {
    font-size: 16px;
    font-style: normal;
    line-height: 20px;
    white-space: nowrap;
  }
}
</style>
