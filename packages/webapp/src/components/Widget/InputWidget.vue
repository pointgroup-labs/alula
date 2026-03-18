<script lang="ts" setup>
import type { Size } from 'bootstrap-vue-next'
import Decimal from 'decimal.js'
import { formatPrice, getZeroCountAfterDecimal, parseFormattedPrice } from '~/utils'

const {
  balance,
  price,
  size = 'md',
  disabled,
  readonly = false,
  fee = 0,
  format = false,
  limit,
  modelValue,
  error,
  rules,
  variant = '',
  reset = false,
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
  error?: string
  variant?: 'cyan' | 'indigo' | 'success' | 'accent'
  reset?: boolean
  symbol?: string
}>()

const emit = defineEmits(['update:modelValue', 'maxHandler'])

const slot = defineSlots()
const { publicKey } = useWalletComposable()

const { assetDecimals } = useMarketActions()

const val = computed({
  get() {
    return modelValue ? String(modelValue) : ''
  },
  set(val) {
    emit('update:modelValue', val)
  },
})

const ruleError = computed(() => {
  if (!rules?.length || !publicKey.value || !val.value) {
    return ''
  }
  for (const rule of rules) {
    const result = rule(val.value)
    if (result !== true) {
      return result
    }
  }
  return ''
})

const displayError = computed(() => error || ruleError.value)

const amountActions = ['25%', '50%', '75%', 'max']
const selectedAmount = ref<string | null>(null)

function max(percent?: string | number) {
  const b = new Decimal(balance)
  const f = new Decimal(fee)
  const result = b.minus(f).toNumber()
  const maxVal = Math.max(Math.min(result, limit || balance), 0) || 0
  const decimals = String(maxVal).includes('e') ? getZeroCountAfterDecimal(maxVal) : null
  let maxAmount = decimals ? maxVal.toFixed(decimals) : String(maxVal)
  const [, dec] = maxAmount.toString().split('.')
  if (!decimals && dec && dec.length > assetDecimals.value) {
    maxAmount = truncatePercent(Number(maxAmount), assetDecimals.value)
  }
  if (percent && percent !== 'max') {
    return Number(maxAmount) * (Number(percent) / 100)
  }
  return maxAmount
}

function handleAmount(percent: string | null) {
  if (!percent || balance <= 0) {
    return
  }
  if (percent === selectedAmount.value) {
    selectedAmount.value = null
    val.value = ''
    return
  }
  selectedAmount.value = percent
  const result = max(percent.replace('%', ''))
  val.value = String(result)
  emit('maxHandler', val.value)
}

const inputDesc = computed(() => {
  if (!price) {
    return
  }
  const stakedSol = format ? parseFormattedPrice(val.value) : Number(val.value)
  const solToUsd = stakedSol * Number(price)
  return `$${formatPrice(solToUsd, 2, 2)}`
})

watch(() => reset, () => {
  val.value = ''
  selectedAmount.value = null
})
</script>

<template>
  <div class="input-widget">
    <div
      v-if="labelLeft || labelRight || slot['label-right']"
      class="input-widget__label"
    >
      <span class="input-widget__label-left">{{ labelLeft }}</span>
      <slot
        v-if="slot['label-right']"
        name="label-right"
      />
      <span
        v-else
        class="input-widget__label-right"
      >
        <span @click="handleAmount('max')">{{ labelRight }}</span> {{ symbol }}
      </span>
    </div>

    <div
      class="input-block"
      :class="[
        variant,
        { active: val && Number(val) > 0, error: displayError },
      ]"
    >
      <div class="input-block__top">
        <template v-if="icon || slot.prepend">
          <img
            v-if="icon"
            :src="icon"
            alt="token icon"
            class="input-block__icon"
          >
          <slot
            v-else
            name="prepend"
          />
        </template>
        <j-input
          v-model="val"
          :size="size"
          placeholder="0.00"
          only-numbers
          :disabled="disabled"
          :readonly="readonly"
          @input="selectedAmount = null"
        />
      </div>
      <div class="input-block__btns">
        <div
          class="select-amount"
          :class="{ 'select-amount--disabled': balance <= 0 }"
        >
          <span
            v-for="value in amountActions"
            :key="value"
            :class="{ active: value === selectedAmount }"
            @click="handleAmount(value)"
          >{{ value }}</span>
        </div>
        <div
          v-if="inputDesc"
          class="amount-to-dollar"
        >
          {{ inputDesc }}
        </div>
      </div>
    </div>

    <div
      v-if="displayError"
      class="input-widget__error"
    >
      {{ displayError }}
    </div>
  </div>
</template>

<style lang="scss">
.input-widget {
  display: flex;
  flex-direction: column;

  --background-color: #{$brand-900};
  --border-color: rgba(0, 211, 238, 0.3);
  --btn-bg: rgba(0, 211, 238, 0.15);
  --active-background-color: #{$brand-700};
  --active-border-color: #{$brand-200};
  --color: #22d3ee;

  .indigo {
    --background-color: #{$indigo-900};
    --border-color: #{$indigo-500};
    --btn-bg: rgba(138, 142, 244, 0.15);
    --active-background-color: #{$indigo-800};
    --active-border-color: #{$indigo-200};
    --color: #{$indigo};
  }

  .success {
    --background-color: rgba(0, 201, 80, 0.03);
    --border-color: rgba(0, 201, 80, 0.3);
    --btn-bg: rgba(0, 201, 80, 0.15);
    --active-border-color: #{$green-500};
    --active-background-color: rgba(0, 201, 80, 0.07);
    --color: #{$success};
  }

  .accent {
    --background-color: rgba(245, 159, 11, 0.03);
    --border-color: rgba(245, 159, 11, 0.3);
    --btn-bg: rgba(245, 159, 11, 0.15);
    --active-background-color: rgba(245, 159, 11, 0.07);
    --active-border-color: #{$orange-500};
    --color: #f59e0b;
  }

  &__label {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 12px;
    color: $text-tertiary;
    margin-bottom: 8px;
  }

  &__label-right {
    font-family: $font-JetBrainsMono;
    user-select: none;

    span {
      border-bottom: 1px dashed $text-tertiary;
      cursor: pointer;
    }
  }

  &__error {
    color: $danger;
    margin: 8px 0 0;
    font-size: 12px;
  }

  .input-block {
    padding: 0;
    background-color: var(--background-color);
    border: 1px solid var(--border-color, $border-primary);
    border-radius: $radius-2xl;
    transition: border-color 0.2s ease;

    &.active {
      border-color: var(--active-border-color);
      background-color: var(--active-background-color);
    }

    &.error {
      --btn-bg: rgb(244 63 94 / 8%);
      --color: $danger;

      background-color: $red-975;
      border-color: $danger;
    }

    &__top {
      display: flex;
      align-items: center;
      padding: 16px;
      gap: 8px;

      .input-group {
        border: none !important;
        background: transparent;
      }

      input {
        height: 100%;
        text-align: right;
        font-family: $font-JetBrainsMono;
        font-weight: 500;
        font-size: 1.4rem;
        color: $text-primary;

        &::placeholder {
          color: $text-tertiary;
          opacity: 0.5;
        }
      }
    }

    &__btns {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0 16px 12px;
    }

    &__icon {
      width: 24px;
      height: 24px;
      flex-shrink: 0;
    }
  }

  .select-amount {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    font-size: 12px;
    color: $text-tertiary;

    &--disabled {
      user-select: none;
      pointer-events: none;
    }

    span {
      padding: 4px 10px;
      font-size: $text-xs;
      text-transform: uppercase;
      border-radius: $radius-sm;
      color: $text-tertiary;
      background-color: color-mix(in oklab, $secondary 60%, transparent);
      transition: all 0.1s ease;
      cursor: pointer;

      &:hover {
        color: $text-primary;
      }

      &.active {
        color: var(--color);
        background-color: var(--btn-bg);
      }
    }
  }

  .amount-to-dollar {
    font-size: 12px;
    font-family: $font-JetBrainsMono;
    color: $text-tertiary;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-left: 16px;
  }
}
</style>
