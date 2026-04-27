<script lang="ts" setup>
import type { BFormInputProps, Size } from 'bootstrap-vue-next'
import { onlyNumber } from '~/utils'

const {
  rules = [],
  size = 'lg',
  lazyRules = false,
  onlyNumbers = false,
  modelValue,
  resetValidation = false,
  forceValidation = false,
  ...props
} = defineProps<{
  size?: Size
  placeholder?: string
  rules?: Array<(val: string | number) => true | string>
  lazyRules?: boolean
  onlyNumbers?: boolean
  modelValue?: string | number
  resetValidation?: boolean
  inputLabel?: string
  forceValidation?: boolean
} & BFormInputProps>()

const emit = defineEmits(['update:modelValue'])
const slots = defineSlots()

const scopeClass = ref()

const inputVal = computed({
  get() {
    return modelValue
  },
  set(val) {
    emit('update:modelValue', val)
  },
})
const errorMessage = ref<string | null>(null)

const computedClasses = computed(() => {
  const classes: string[] = []
  if (errorMessage.value) {
    classes.push('j-input--error')
  }
  return classes.join(' ')
})

function validate() {
  if (!inputVal.value) {
    errorMessage.value = null
    return
  }
  for (const rule of rules) {
    const result = rule(inputVal.value)
    if (result !== true) {
      errorMessage.value = result as string
      return false
    }
  }
  errorMessage.value = null
}

function handleBlur() {
  if (lazyRules) {
    validate()
  }
}

// watch(value, () => {
//   emit('update:modelValue', value.value)
// }, { flush: 'post' })

// watch(() => modelValue, (val) => {
//   value.value = val
// })

watch(inputVal, () => {
  if (!lazyRules) {
    validate()
  }
})

watch(() => resetValidation, (val) => {
  if (val) {
    errorMessage.value = null
  }
})

watch(() => forceValidation, (val) => {
  if (val) {
    nextTick(() => validate())
  }
})

function handleClick(e: Event) {
  if (e.target instanceof HTMLInputElement) {
    return
  }
  focusInput(`.${scopeClass.value}`)
}

onMounted(() => {
  scopeClass.value = `j-input-${Math.random().toString(36).slice(2, 11)}`
})
</script>

<template>
  <div
    class="j-input"
    :class="scopeClass"
    @click="handleClick"
  >
    <div
      v-if="slots?.label"
      class="j-input__label"
    >
      <slot name="label" />
    </div>
    <b-input-group
      :class="computedClasses"
      :size="size"
    >
      <div class="input-wrapper">
        <div
          v-if="inputLabel"
          class="input-wrapper__label"
        >
          {{ inputLabel }}
        </div>
        <b-form-input
          v-bind="props"
          v-model="inputVal"
          :placeholder="placeholder"
          :type="type"
          autocomplete="off"
          @keypress="onlyNumbers && onlyNumber($event)"
          @blur="handleBlur"
        />
      </div>
      <template
        v-if="slots.prepend"
        #prepend
      >
        <div class="j-input__prepend">
          <slot name="prepend" />
        </div>
      </template>
      <template
        v-if="slots.append"
        #append
      >
        <div class="j-input__append">
          <slot name="append" />
        </div>
      </template>
    </b-input-group>

    <transition name="fade-bottom">
      <div
        v-if="errorMessage"
        class="validate-label"
      >
        {{ errorMessage }}
      </div>
    </transition>

    <div
      v-if="slots.description"
      class="j-input__desc"
      :class="{ hide: errorMessage }"
    >
      <slot name="description" />
    </div>
  </div>
</template>

<style lang="scss" scoped>
/* Transition class for Vue */
.fade-bottom-enter-active,
.fade-bottom-leave-active {
  transition:
    opacity $transition-base ease,
    transform $transition-base ease;
}

.fade-bottom-enter-from,
.fade-bottom-leave-to {
  opacity: 0;
  transform: translateY(-20px);
}

.fade-bottom-enter-to,
.fade-bottom-leave-from {
  opacity: 1;
  transform: translateY(0);
}
</style>
