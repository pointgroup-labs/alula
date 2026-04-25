<script lang="ts" setup>
import type { SwapRoute } from '@alula/client-sdk'
import aquaLogo from '~/assets/img/providers/aqua-logo.png'
import soroswapLogo from '~/assets/img/providers/soroswap-logo.jpg'
import { bigintToNumber, shortenNumber } from '~/utils'

const {
  routes,
  selectedRoute,
  pinnedRouteKey,
  fromSymbol,
  fromAmount,
  toSymbol,
  loading,
} = defineProps<{
  routes: SwapRoute[]
  selectedRoute?: SwapRoute
  pinnedRouteKey?: string
  /** Symbol of the input token — drives the dropdown's query header. */
  fromSymbol?: string
  /** Human-units input amount — drives the dropdown's query header. */
  fromAmount?: number
  toSymbol?: string
  loading?: boolean
}>()

const emit = defineEmits<{
  pin: [key: string | undefined]
}>()

// Map provider names to their brand logos so the picker matches the look used
// elsewhere (multiply window, old `<provider-select>`).
const providerIcons: Record<string, string> = {
  Aquarius: aquaLogo,
  Soroswap: soroswapLogo,
}

function iconFor(name: string): string | undefined {
  return providerIcons[name]
}

function fmtAmount(value: number, decimals = 6): string {
  if (!Number.isFinite(value)) {
    return '—'
  }
  return value > 1000 ? shortenNumber(value) : value.toFixed(Math.min(decimals, 6))
}

function routeOutHuman(route: SwapRoute): number {
  // Decimal-backed conversion — `Number(bigint) / 10 ** decimals` silently
  // loses precision for outputs near MAX_SAFE_INTEGER (relevant for high-
  // decimal tokens or very large quotes), and the delta-vs-best comparison
  // would compound the error.
  return Number(bigintToNumber(route.expectedAmountOut, route.toTokenDecimals))
}

function fmtOut(route: SwapRoute): string {
  return fmtAmount(routeOutHuman(route), route.toTokenDecimals)
}

// Delta vs the top-ranked (best) route, expressed as a negative percentage so
// the user immediately sees the cost of choosing a non-best provider.
function deltaPctVsBest(route: SwapRoute): number | undefined {
  const best = routes[0]
  if (!best || best.key === route.key) {
    return
  }
  const bestOut = routeOutHuman(best)
  const thisOut = routeOutHuman(route)
  if (!Number.isFinite(bestOut) || bestOut <= 0 || !Number.isFinite(thisOut)) {
    return
  }
  return ((thisOut - bestOut) / bestOut) * 100
}

const isAuto = computed(() => pinnedRouteKey == null)

// Single-line context header so the user knows which input these routes were
// quoted against. Hidden when we don't have enough info to render it.
const queryLine = computed<string | undefined>(() => {
  if (!fromSymbol || !toSymbol) {
    return
  }
  if (fromAmount && fromAmount > 0) {
    return `${fmtAmount(fromAmount)} ${fromSymbol} → ${toSymbol}`
  }
  return `${fromSymbol} → ${toSymbol}`
})

function selectAuto() {
  emit('pin')
}

function selectRoute(route: SwapRoute) {
  emit('pin', route.key)
}
</script>

<template>
  <div class="route-picker">
    <j-popover
      position="bottom"
      placement="bottom-end"
      :teleport-to-body="false"
      close-popup
    >
      <template #target="{ active }">
        <button
          type="button"
          class="route-picker__trigger"
          :class="{ 'route-picker__trigger--open': active }"
          :disabled="routes.length === 0 && !loading"
        >
          <span
            v-if="loading && !selectedRoute"
            class="route-picker__placeholder"
          >Searching routes…</span>

          <template v-else-if="selectedRoute">
            <img
              v-if="iconFor(selectedRoute.providerName)"
              :src="iconFor(selectedRoute.providerName)"
              :alt="selectedRoute.providerName"
              class="route-picker__provider-icon"
            >
            <span class="route-picker__provider-name">{{ selectedRoute.providerName }}</span>
            <span
              v-if="isAuto"
              class="route-picker__auto-badge"
            >AUTO</span>
          </template>

          <span
            v-else
            class="route-picker__placeholder"
          >No route</span>

          <i-app-chevron-down
            class="route-picker__arrow"
            :class="{ 'route-picker__arrow--active': active }"
          />
        </button>
      </template>

      <div class="route-picker__menu">
        <div class="route-picker__menu-header">
          <span class="route-picker__menu-title">Choose route</span>
          <span
            v-if="queryLine"
            class="route-picker__menu-query"
          >{{ queryLine }}</span>
        </div>

        <button
          type="button"
          class="route-picker__option route-picker__option--auto"
          :class="{ 'route-picker__option--active': isAuto }"
          @click="selectAuto"
        >
          <span class="route-picker__option-label">
            <span class="route-picker__option-title">Auto</span>
            <span class="route-picker__option-sub">Use the best-priced route</span>
          </span>
          <span
            v-if="isAuto"
            class="route-picker__option-check"
          >✓</span>
        </button>

        <div
          v-if="routes.length === 0"
          class="route-picker__empty"
        >
          {{ loading ? 'Searching…' : 'No routes available for this pair.' }}
        </div>

        <button
          v-for="(route, idx) in routes"
          :key="route.key"
          type="button"
          class="route-picker__option"
          :class="{ 'route-picker__option--active': route.key === selectedRoute?.key }"
          @click="selectRoute(route)"
        >
          <img
            v-if="iconFor(route.providerName)"
            :src="iconFor(route.providerName)"
            :alt="route.providerName"
            class="route-picker__option-icon"
          >
          <span class="route-picker__option-label">
            <span class="route-picker__option-title">
              {{ route.providerName }}
              <span
                v-if="idx === 0"
                class="route-picker__option-tag route-picker__option-tag--best"
              >best</span>
              <span
                v-else-if="deltaPctVsBest(route) != null"
                class="route-picker__option-tag route-picker__option-tag--delta"
              >{{ deltaPctVsBest(route)!.toFixed(2) }}%</span>
            </span>
            <span class="route-picker__option-sub">
              ≈ {{ fmtOut(route) }} {{ toSymbol ?? '' }}
            </span>
          </span>
          <span
            v-if="route.key === selectedRoute?.key && !isAuto"
            class="route-picker__option-check"
          >✓</span>
        </button>
      </div>
    </j-popover>
  </div>
</template>

<style lang="scss">
// Picker fills its parent row; the trigger pill is content-sized and pushed
// to the right edge by the picker's own `justify-content: flex-end`. We do
// NOT override `.popover-target`'s width — Popper anchors the menu to the
// popover-target's bounding box, so widening that wrapper makes the menu
// open under the wrapper's center (= the middle of the row) instead of under
// the trigger pill. Keeping popover-target at its base `width: fit-content`
// (set by JPopover) preserves the correct anchor.
.route-picker {
  display: flex;
  width: 100%;
  justify-content: flex-end;

  &__trigger {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    background-color: color-mix(in oklab, $navi-700 80%, transparent);
    border: 1px solid $border-primary;
    border-radius: $radius-full;
    color: $text-primary;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition:
      border-color $transition-base ease,
      background-color $transition-base ease;

    &:hover:not(:disabled) {
      border-color: $navi-300;
      background-color: $navi-700;
    }

    &:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }

    &--open {
      border-color: $navi-200;
    }
  }

  &__provider-icon {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    object-fit: contain;
  }

  &__provider-name {
    color: $text-primary;
    font-weight: 600;
  }

  &__auto-badge {
    font-size: 10px;
    font-weight: 700;
    color: $cyan;
    letter-spacing: 0.06em;
    background-color: color-mix(in oklab, $cyan 15%, transparent);
    border: 1px solid color-mix(in oklab, $cyan 35%, transparent);
    border-radius: 4px;
    padding: 1px 5px;
  }

  &__placeholder {
    color: $text-tertiary;
  }

  &__arrow {
    color: $text-tertiary;
    transition: transform $transition-base ease;
    &--active {
      transform: rotate(180deg);
    }
  }

  // No width constraints on the menu — let it size to its content. With
  // `placement="bottom-end"` the right edge already anchors to the trigger's
  // right edge, so a content-sized menu is right-aligned and can't overflow.
  &__menu {
    padding: 6px;
    background-color: $bg-card;
    border: 1px solid $border-primary;
    border-radius: $radius-lg;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  &__menu-header {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px 10px 8px;
    margin-bottom: 4px;
    border-bottom: 1px solid $border-primary;
  }

  &__menu-title {
    font-size: 11px;
    font-weight: 600;
    color: $text-tertiary;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  &__menu-query {
    font-size: 12px;
    color: $text-secondary;
    font-family: $font-JetBrainsMono;
    word-break: break-word;
    line-height: 1.35;
  }

  &__option {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    background: none;
    border: none;
    border-radius: $radius-md;
    cursor: pointer;
    text-align: left;
    color: $text-primary;
    transition: background-color 0.12s ease;

    &:hover {
      background-color: $navi-600;
    }

    &--active {
      background-color: color-mix(in oklab, $brand-700 25%, transparent);
    }
  }

  &__option-icon {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    object-fit: contain;
    flex-shrink: 0;
  }

  &__option-label {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    line-height: 1.2;
  }

  &__option-title {
    font-size: 13px;
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  &__option-tag {
    font-size: 10px;
    font-weight: 600;
    color: $text-tertiary;
    background-color: color-mix(in oklab, $navi-600 60%, transparent);
    border-radius: 4px;
    padding: 1px 5px;
    text-transform: uppercase;
    letter-spacing: 0.04em;

    &--best {
      color: $cyan;
      background-color: color-mix(in oklab, $cyan 15%, transparent);
    }

    &--delta {
      color: $text-tertiary;
    }
  }

  &__option-sub {
    font-size: 11px;
    color: $text-tertiary;
    font-family: $font-JetBrainsMono;
  }

  &__option-check {
    color: $cyan;
    font-weight: 700;
  }

  &__empty {
    font-size: 12px;
    color: $text-tertiary;
    padding: 8px 10px;
  }
}
</style>
