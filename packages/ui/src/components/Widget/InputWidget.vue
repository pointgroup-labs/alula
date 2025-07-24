<script lang="ts" setup>
import type { Size } from 'bootstrap-vue-next'
import Decimal from 'decimal.js'
import { focusInput, formatPrice, parseFormattedPrice } from '~/utils'

const {
  balance,
  price,
  size = 'lg',
  disabled,
  readonly = false,
  fee = 0,
  format = false,
  limit,
  modelValue,
} = defineProps<{
  size?: Size
  balance: number
  description?: string
  price?: number
  limit?: number
  disabled?: boolean
  labelLeft?: string
  labelRight?: string
  icon?: string
  readonly?: boolean
  fee?: number
  format?: boolean
  rules?: Array<(val: string | number) => true | string>
  modelValue?: string | number
}>()

const emit = defineEmits(['update:modelValue', 'maxHandler'])

const slot = defineSlots()

const wallet = useWallet()

const val = computed({
  get() {
    return modelValue ? String(modelValue) : ''
  },
  set(val) {
    emit('update:modelValue', val)
  },
})
const resetValidation = ref(false)

function max() {
  const b = new Decimal(balance)
  const f = new Decimal(fee)
  const result = b.minus(f).toNumber()
  val.value = String(Math.max(Math.min(result, limit || balance), 0))
  resetValidation.value = true
  nextTick(() => {
    resetValidation.value = false
    emit('maxHandler', val.value)
  })
}

const inputDesc = computed(() => {
  if (!val.value || !price) {
    return
  }
  const stakedSol = format ? parseFormattedPrice(val.value) : Number(val.value)
  const solToUsd = stakedSol * Number(price)
  return `$${formatPrice(solToUsd, 2, 2)}`
})

function handleClick(e: any) {
  const target = e.target
  if (target.closest('.j-input__label') || target.closest('.j-input__desc')) {
    return
  }
  focusInput('.j-input')
}

const forceValidation = ref(false)

watch(() => balance, () => {
  forceValidation.value = true
  nextTick(() => {
    forceValidation.value = false
  })
})
</script>

<template>
  <j-input
    v-model="val"
    class="input-widget"
    :size="size"
    placeholder="0.00"
    only-numbers
    :force-validation="forceValidation"
    :reset-validation="resetValidation"
    :disabled="disabled"
    :readonly="readonly"
    :rules="rules"
    @click="handleClick"
  >
    <template #label>
      <span>{{ labelLeft }}</span>
      <slot
        v-if="slot['label-right']"
        name="label-right"
      />
      <span
        v-else
        class="balance"
      >
        {{ labelRight }}
      </span>
    </template>
    <template #description>
      <div class="price-label">
        {{ inputDesc }}
      </div>
    </template>

    <template
      v-if="icon"
      #prepend
    >
      <img
        :src="icon"
        alt="token icon"
        class="j-input__icon"
      >
    </template>
    <template #append>
      <j-btn
        :disabled="!wallet.publicKey"
        variant="accent"
        size="sm"
        class="j-input__btn"
        @click="max"
      >
        MAX
      </j-btn>
    </template>
  </j-input>
</template>

<style lang="scss">
.input-widget {
  .balance {
    font-size: 14px;
    margin-left: 16px;
    opacity: 0.8;
  }

  .price-label {
    height: 14px;
    text-align: right;
  }

  .input-group {
    background-color: $neutral-2;
    border-radius: $spacing-12;
    border: none;
  }

  .j-input__btn {
    border-radius: 4px;
    width: max-content;
    text-transform: uppercase;

    .btn-content {
      transform: none;
    }
  }
}
</style>
