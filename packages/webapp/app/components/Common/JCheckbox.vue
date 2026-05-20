<script lang="ts" setup>
const {
  bold,
  modelValue,
  size = 'lg',
  ...props
} = defineProps<{
  bold?: boolean
  size?: 'sm' | 'md' | 'lg'
  modelValue: any
  color?: string
  bgColor?: string
}>()

const emit = defineEmits(['update:modelValue'])

const value = computed({
  get() {
    return modelValue
  },
  set(val) {
    emit('update:modelValue', val)
  },
})
</script>

<template>
  <b-form-checkbox
    v-model="value"
    :size="size"
    :value="true"
    :unchecked-value="false"
    class="j-checkbox"
    :class="{ bold }"
    :state="null"
    v-bind="props"
    :style="{ '--checkbox-color': color, '--checkbox-bg-color': bgColor }"
  >
    <slot />
  </b-form-checkbox>
</template>
