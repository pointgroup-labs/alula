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
  variant?: 'supply' | 'borrow' | 'success'
  reset?: boolean
}>()

const emit = defineEmits(['update:modelValue', 'maxHandler'])

const slot = defineSlots()
const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

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
  if (!rules?.length || !publicKey.value) {
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
  if (!percent) { return }
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
        {{ labelRight }}
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
        <div class="select-amount">
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

  --background-color: rgba(0, 211, 238, 0.03);
  --border-color: rgba(0, 211, 238, 0.3);
  --color: #22d3ee;

  .borrow {
    --background-color: rgba(99, 102, 241, 0.05);
    --border-color: rgba(99, 102, 241, 0.3);
    --color: #8a8df4;
  }

  .success {
    --background-color: rgba(0, 201, 80, 0.03);
    --border-color: rgba(0, 201, 80, 0.3);
    --color: #00c950;
  }

  &__label {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 12px;
    color: $muted-foreground;
    margin-bottom: 8px;
  }

  &__label-right {
    font-family: $font-JetBrainsMono;
  }

  &__error {
    color: #f43f5e;
    margin: 8px 0 0;
    font-size: 12px;
  }

  .input-block {
    padding: 0;
    background-color: color-mix(in oklab, $new-secondary 30%, transparent);
    border: 1px solid $border-color;
    border-radius: 14px;
    transition: border-color 0.2s ease;

    &.active {
      background-color: var(--background-color);
      border-color: var(--border-color);
    }

    &.error {
      background-color: rgb(244 63 94 / 10%);
      border-color: #f43f5e;
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
        color: $foreground;

        &::placeholder {
          color: $muted-foreground;
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
    color: $muted-foreground;

    span {
      padding: 4px 10px;
      font-size: 11px;
      text-transform: uppercase;
      border-radius: 6px;
      color: $muted-foreground;
      background-color: color-mix(in oklab, $new-secondary 60%, transparent);
      transition: all 0.1s ease;
      cursor: pointer;

      &:hover {
        color: $foreground;
      }

      &.active {
        color: var(--color);
        background-color: rgba(0, 211, 238, 0.15);
      }
    }
  }

  .amount-to-dollar {
    font-size: 12px;
    font-family: $font-JetBrainsMono;
    color: $muted-foreground;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-left: 16px;
  }
}
</style>
