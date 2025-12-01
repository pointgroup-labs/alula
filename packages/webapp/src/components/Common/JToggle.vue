<script lang="ts" setup>
const {
  modelValue,
  disabled = false,
} = defineProps<{
  modelValue: boolean
  disabled?: boolean
  color?: string
}>()

const emit = defineEmits(['update:modelValue'])
const slots = defineSlots()

const value = ref(false)

function toggleHandler() {
  if (disabled) {
    return
  }
  value.value = !value.value
}

watch(value, () => {
  emit('update:modelValue', value.value)
})

watch(() => modelValue, (v) => {
  value.value = v
}, { immediate: true })
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
  gap: $spacing-8;
  cursor: pointer;

  &__label {
    font-size: 16px;
    font-style: normal;
    line-height: 20px;
    white-space: nowrap;
  }
}
</style>
