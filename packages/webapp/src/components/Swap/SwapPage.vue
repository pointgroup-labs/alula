<script lang="ts" setup>
import type { SwapTokenOption } from '~/hooks/swap/use-swap'
import { bigintToNumber, formatPrice, shortenNumber, truncatePercent } from '~/utils'

definePageMeta({
  layout: 'default',
})

const {
  tokens,
  fromToken,
  toToken,
  fromBalance,
  toBalance,
  amount,
  amountNumber,
  slippage,
  routes,
  selectedRoute,
  pinnedRouteKey,
  pinRoute,
  preview,
  publicKey,
  loading,
  error,
  submitting,
  isReady,
  flip,
  quote,
  submit,
} = useSwap()

const connectionStore = useConnectionStore()
const wallet = useWallet()

const isConnected = computed(() => !!publicKey.value)

const isLoadingBalances = computed(() => !wallet.balances && wallet.isloadingBalances)

const isFlip = ref(false)

function flipHandler() {
  isFlip.value = !isFlip.value
  flip()
}

// Hide the From-token from the To-picker (and vice-versa) so the user can't pick
// the same token on both sides — the SDK would error out anyway with "Pick two
// different tokens", but it's cheaper to hide the impossible choice up front.
const fromTokenOptions = computed(() =>
  tokens.value.filter(t => t.tokenAddress !== toToken.value?.tokenAddress && isValidPair(t.symbol, toToken.value?.symbol ?? '')),
)
const toTokenOptions = computed(() =>
  tokens.value.filter(t => t.tokenAddress !== fromToken.value?.tokenAddress && isValidPair(t.symbol, fromToken.value?.symbol ?? '')),
)

const isFromTokenDialogOpen = ref(false)
const isToTokenDialogOpen = ref(false)

function dialogsHandler(side: 'from' | 'to') {
  if (side === 'from') {
    isFromTokenDialogOpen.value = !isFromTokenDialogOpen.value
  } else {
    isToTokenDialogOpen.value = !isToTokenDialogOpen.value
  }
}

// Human-readable estimated `to` amount and its USD shadow.
// `bigintToNumber` routes through Decimal.js so we don't lose precision the
// way `Number(bigint) / 10 ** decimals` does for large values or high-decimal
// tokens. Returns a string already in human units.
function fmtAmount(value: bigint | undefined, decimals: number): string {
  if (value == null) {
    return '—'
  }
  const human = Number(bigintToNumber(value, decimals))
  if (!Number.isFinite(human)) {
    return '—'
  }
  return human > 1000 ? shortenNumber(human) : human.toFixed(Math.min(6, decimals))
}

const expectedReceiveHuman = computed(() => {
  if (!preview.value || !toToken.value) {
    return 0
  }
  return Number(bigintToNumber(preview.value.expectedAmountOut, toToken.value.tokenDecimals))
})
const expectedReceiveDisplay = computed(() =>
  preview.value && toToken.value
    ? fmtAmount(preview.value.expectedAmountOut, toToken.value.tokenDecimals)
    : '',
)
const expectedReceiveUsd = computed(() =>
  expectedReceiveHuman.value * (toToken.value?.price || 0),
)

const minReceiveDisplay = computed(() =>
  preview.value && toToken.value
    ? fmtAmount(preview.value.minAmountOut, toToken.value.tokenDecimals)
    : '',
)

// SDK already returns realized output/in price, but we round it for display.
const rateLine = computed(() => {
  if (!preview.value || !fromToken.value || !toToken.value) {
    return ''
  }
  return `1 ${fromToken.value.symbol} ≈ ${formatPrice(preview.value.spotPrice, 2, 6)} ${toToken.value.symbol}`
})

// Removed: a `providerName` computed used to drive a separate "Routed via" row.
// The route picker now shows the provider front-and-center, so the extra row
// was redundant and we deleted it from the template.

const insufficientBalance = computed(() =>
  amountNumber.value > 0 && amountNumber.value > fromBalance.value,
)
const sameToken = computed(() =>
  !!fromToken.value && !!toToken.value
  && fromToken.value.tokenAddress === toToken.value.tokenAddress,
)

const buttonLabel = computed(() => {
  if (!isConnected.value) {
    return 'Connect wallet'
  }
  if (!isReady.value) {
    return 'Loading markets…'
  }
  if (sameToken.value) {
    return 'Pick two different tokens'
  }
  if (amountNumber.value <= 0) {
    return 'Enter an amount'
  }
  if (insufficientBalance.value) {
    return `Insufficient ${fromToken.value?.symbol ?? ''} balance`
  }
  if (loading.value) {
    return 'Quoting…'
  }
  if (error.value) {
    return 'Retry quote'
  }
  if (!preview.value) {
    return 'Quote'
  }
  return `Swap ${fromToken.value?.symbol ?? ''} → ${toToken.value?.symbol ?? ''}`
})

const buttonDisabled = computed(() => {
  if (sameToken.value || insufficientBalance.value) {
    return true
  }
  // Allow click while loading? No — but allow click when quote errored so the
  // user can retry without re-typing.
  if (error.value) {
    return false
  }
  return false
})

async function onSubmit() {
  if (!isConnected.value) {
    await connectionStore.connectWallet?.()
    return
  }
  if (amountNumber.value <= 0) {
    focusInput('.swap-card__amount-input')
    return
  }
  // If quote previously errored or hasn't fired, retry it from the button.
  if (!preview.value) {
    await quote()
    return
  }
  await submit()
}

function pick(target: 'from' | 'to', token: SwapTokenOption) {
  if (target === 'from') {
    if (toToken.value?.tokenAddress === token.tokenAddress) {
      // User picked the current `to` token — flip rather than create a duplicate.
      toToken.value = fromToken.value
    }
    fromToken.value = token
  } else {
    if (fromToken.value?.tokenAddress === token.tokenAddress) {
      fromToken.value = toToken.value
    }
    toToken.value = token
  }
}

const amountRules = [
  (value: string | number) => {
    if (value === '' || value == null) {
      return true
    }
    const n = Number(value)
    if (!Number.isFinite(n) || n < 0) {
      return 'Amount must be a positive number'
    }
    if (n > fromBalance.value) {
      return 'Amount exceeds wallet balance'
    }
    return true
  },
]
</script>

<template>
  <main class="swap-page container">
    <div class="swap-card">
      <div class="swap-card__header">
        <div class="swap-card__header-titles">
          <h1>Swap</h1>
          <span class="swap-card__subtitle">
            Instantly swap supported assets at the best on-chain rate.
          </span>
        </div>
        <slippage-select v-model="slippage" />
      </div>

      <input-widget
        v-model="amount"
        :fee="XLM_NATIVE_RESERVE"
        :balance="fromBalance"
        :limit="fromBalance"
        :price="fromToken?.price ?? 0"
        :symbol="fromToken?.symbol"
        label-left="Wallet balance"
        :label-right="formatPrice(fromBalance ?? 0, 0, 4)"
        :rules="amountRules"
        variant="cyan"
        class="swap-card__amount-input"
      >
        <template #prepend>
          <button
            v-if="fromToken"
            type="button"
            class="select-pool-btn swap-card__token-btn"
            :aria-label="`Choose token to swap from (current: ${fromToken?.symbol ?? 'none'})`"
            @click="dialogsHandler('from')"
          >
            <img
              v-if="fromToken?.icon"
              :src="fromToken.icon"
              :alt="fromToken.symbol"
            >
            <span class="swap-card__token-btn-symbol">{{ fromToken?.symbol }}</span>
            <i-app-chevron-down
              class="arrow-icon"
            />
          </button>

        </template>
      </input-widget>

      <button
        type="button"
        class="swap-card__flip"
        :disabled="!isReady"
        :aria-label="`Switch ${fromToken?.symbol ?? ''} and ${toToken?.symbol ?? ''}`"
        :style="{ transform: `rotate(${isFlip ? 180 : 0}deg)` }"
        @click="flipHandler"
      >
        <i-app-line-arrow-right class="swap-card__flip-icon" />
        <i-app-line-arrow-right class="swap-card__flip-icon top" />
      </button>

      <div class="swap-card__receive">
        <div class="swap-card__receive-label">
          <span>You receive</span>
          <span class="text-num">
            {{ formatPrice(toBalance ?? 0, 0, 4) }} {{ toToken?.symbol }}
          </span>
        </div>
        <div class="swap-card__receive-row">
          <button
            v-if="toToken"
            type="button"
            class="select-pool-btn swap-card__token-btn"
            :aria-label="`Choose token to swap to (current: ${toToken?.symbol ?? 'none'})`"
            @click="dialogsHandler('to')"
          >
            <img
              v-if="toToken?.icon"
              :src="toToken.icon"
              :alt="toToken.symbol"
            >
            <span class="swap-card__token-btn-symbol">{{ toToken?.symbol }}</span>
            <i-app-chevron-down
              class="arrow-icon"
            />
          </button>

          <div class="swap-card__receive-amount">
            <span
              v-if="loading && !preview"
              class="swap-card__receive-placeholder"
              aria-label="Quoting"
              aria-busy="true"
            >…</span>
            <span
              v-else-if="expectedReceiveDisplay"
              class="swap-card__receive-value"
            >{{ expectedReceiveDisplay }}</span>
            <span
              v-else
              class="swap-card__receive-placeholder"
            >0.00</span>

            <span class="swap-card__receive-usd">
              <template v-if="preview && toToken?.price">
                ≈ ${{ formatPrice(expectedReceiveUsd, 2, 2) }}
              </template>
            </span>
          </div>
        </div>
      </div>

      <div class="swap-card__quote-rows">
        <div
          v-if="error"
          class="swap-card__error"
        >
          {{ error }}
        </div>
        <div class="swap-card__quote-row">
          <span>Rate</span>
          <span class="text-num">{{ rateLine || '—' }}</span>
        </div>
        <div class="swap-card__quote-row">
          <span>Min received <span class="text-tertiary">({{ truncatePercent(Number(slippage), 2) }}% slippage)</span></span>
          <span class="text-num">
            <template v-if="preview && toToken">
              {{ minReceiveDisplay }} {{ toToken.symbol }}
            </template>
            <template v-else>—</template>
          </span>
        </div>
        <div class="swap-card__quote-row">
          <span>Route</span>
          <route-picker
            :routes="routes"
            :selected-route="selectedRoute"
            :pinned-route-key="pinnedRouteKey"
            :from-symbol="fromToken?.symbol"
            :from-amount="amountNumber"
            :to-symbol="toToken?.symbol"
            :loading="loading"
            @pin="pinRoute"
          />
        </div>
      </div>

      <market-action-btn
        class="swap-card__submit"
        size="lg"
        :loading="submitting || loading || isLoadingBalances"
        :disabled="buttonDisabled"
        :pool="toToken && {
          name: `${toToken?.symbol}:${toToken?.assetIssuer}`,
          token_symbol: toToken?.tokenSymbol,
        }"
        @click-handler="onSubmit"
      >
        {{ buttonLabel }}
      </market-action-btn>
    </div>

    <div
      v-if="selectedRoute?.providerName"
      class="swap-powered"
    >
      <span>Swap is powered by</span> {{ selectedRoute?.providerName }}
    </div>
  </main>

  <swap-select-asset-dialog
    v-model="isFromTokenDialogOpen"
    :tokens="fromTokenOptions"
    :active-token="fromToken"
    @pick-token="(token) => pick('from', token)"
  />

  <swap-select-asset-dialog
    v-model="isToTokenDialogOpen"
    :tokens="toTokenOptions"
    :active-token="toToken"
    @pick-token="(token) => pick('to', token)"
  />
</template>

<style lang="scss">
.swap-page {
  align-items: center;
  padding: 32px 16px 64px;

  .swap-powered {
    font-size: 14px;
    color: $text-secondary;

    span {
      color: $text-tertiary;
    }
  }
}

.swap-card {
  width: 100%;
  max-width: 480px;
  background-color: $bg-card;
  border: 1px solid $border-secondary;
  border-radius: 12px;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;

  &__header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid $border-primary;
  }

  &__header-titles {
    display: flex;
    flex-direction: column;
    gap: 4px;

    h1 {
      font-size: 20px;
      font-weight: 700;
      color: $navi-25;
      margin: 0;
      line-height: 24px;
    }
  }

  &__subtitle {
    font-size: 12px;
    color: $text-tertiary;
  }

  &__amount-input {
    .input-block__top {
      gap: 12px;
    }
  }

  &__token-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    // padding: 6px 10px;
    // background-color: color-mix(in oklab, $navi-700 80%, transparent);
    // border: 1px solid $border-primary;
    // border-radius: $radius-full;
    cursor: pointer;
    font: inherit;
    color: inherit;
    appearance: none;
    transition:
      border-color $transition-base ease,
      background-color $transition-base ease;

    &:hover {
      border-color: $navi-300;
      background-color: $navi-700;
    }

    img {
      width: 22px;
      height: 22px;
      border-radius: 50%;
    }

    .arrow-icon {
      color: $text-tertiary;
      transition: transform $transition-base ease;
      &--active {
        transform: rotate(180deg);
      }
    }
  }

  &__token-btn-symbol {
    font-size: 14px;
    font-weight: 600;
    color: $text-primary;
  }

  &__flip {
    align-self: center;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background-color: $navi-600;
    border: 1px solid $border-primary;
    padding: 0;
    color: $text-secondary;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    margin: -8px 0;
    transition:
      background-color $transition-base ease,
      transform $transition-base ease,
      color $transition-base ease;

    &:hover:not(:disabled) {
      background-color: $navi-450;
      color: $text-primary;
    }

    &:disabled {
      opacity: 0.4;
      cursor: not-allowed;
    }
  }

  &__flip-icon {
    transform: rotate(-90deg) translatey(1px);
    width: 14px;
    height: 14px;

    &.top {
      transform: rotate(90deg) translatey(1px);
    }
  }

  &__receive {
    display: flex;
    flex-direction: column;
    gap: 8px;
    background-color: color-mix(in oklab, $navi-700 60%, transparent);
    border: 1px solid $border-primary;
    border-radius: $radius-2xl;
    padding: 14px 16px;

    .swap-card__receive-row {
      min-height: 52px;
    }
  }

  &__receive-label {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 12px;
    color: $text-tertiary;
  }

  &__receive-row {
    min-height: 18px;
    display: flex;
    align-items: center;
    gap: 12px;
  }

  &__receive-amount {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    font-family: $font-JetBrainsMono;
  }

  &__receive-placeholder {
    color: $text-tertiary;
    opacity: 0.5;
    font-size: 22px;
    font-weight: 500;
  }

  &__receive-amount > span.swap-card__receive-value {
    font-size: 22px;
    font-weight: 500;
    color: $text-primary;
  }

  &__receive-usd {
    font-size: 12px;
    color: $text-tertiary;
    min-height: 16px;
  }

  &__quote-rows {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 4px 0;
    border-top: 1px solid $border-primary;
  }

  &__error {
    font-size: 12px;
    line-height: 1.4;
    color: $danger;
    background-color: color-mix(in oklab, $danger 12%, transparent);
    border: 1px solid color-mix(in oklab, $danger 35%, transparent);
    border-radius: $radius-md;
    padding: 8px 10px;
    word-break: break-word;
  }

  &__quote-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 12px;
    color: $text-tertiary;
    gap: 12px;

    .text-num {
      color: $text-primary;
      font-family: $font-JetBrainsMono;
    }
  }

  &__submit {
    margin-top: 4px;
    width: 100%;
  }
}
</style>
