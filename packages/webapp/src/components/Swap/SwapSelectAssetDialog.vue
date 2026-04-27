<script lang="ts" setup>
import type { SwapTokenOption } from '~/hooks/swap/use-swap'
import { formatPrice, shortenAddress } from '~/utils'

const {
  tokens = [],
  activeToken,
} = defineProps<{
  tokens?: SwapTokenOption[]
  activeToken?: SwapTokenOption
}>()

const emits = defineEmits(['pickToken'])

const dialog = defineModel({ default: false })
const search = ref('')

function tokenReference(token: SwapTokenOption): string {
  if (token.isNative) {
    return 'Native asset'
  }

  const reference = token.assetIssuer || token.tokenAddress
  return token.assetIssuer
    ? `Issuer ${shortenAddress(reference, 4)}`
    : shortenAddress(reference, 4)
}

function tokenPrice(token: SwapTokenOption): string {
  if (token.price <= 0) {
    return 'Unpriced'
  }

  return `$${formatPrice(token.price, 2, token.price < 1 ? 4 : 2)}`
}

function select(token: SwapTokenOption) {
  emits('pickToken', token)
  dialog.value = false
}

const filteredTokens = computed(() =>
  tokens.filter((t) => {
    const query = search.value.toLowerCase().trim()
    if (!query) {
      return true
    }

    return [t.symbol, t.name, t.tokenAddress, t.assetIssuer]
      .filter(Boolean)
      .some(value => value!.toLowerCase().includes(query))
  }))
</script>

<template>
  <select-entity-dialog
    v-model="dialog"
    v-model:search="search"
    search-placeholder="Search token"
  >
    <div
      v-if="filteredTokens.length > 0"
      class="swap-tokens-list"
    >
      <div
        v-for="token in filteredTokens"
        :key="token.tokenAddress"
        class="swap-token"
        :class="{ 'swap-token--selected': token.tokenAddress === activeToken?.tokenAddress }"
        @click="select(token)"
      >
        <img
          :src="token.icon"
          alt="asset icon"
          class="asset-icon"
        >
        <div class="asset-data">
          <div class="asset-data__top-row">
            <div class="asset-data__symbol">{{ token.symbol }}</div>
            <div
              v-if="token.tokenAddress === activeToken?.tokenAddress"
              class="asset-data__badge"
            >
              Selected
            </div>
          </div>
          <div class="asset-data__name">{{ token.name }}</div>
          <div class="asset-data__meta">{{ tokenReference(token) }}</div>
        </div>
        <div class="asset-price">{{ tokenPrice(token) }}</div>
      </div>
    </div>
    <div
      v-else
      class="no-data"
    >
      No assets to swap
    </div>
  </select-entity-dialog>
</template>

<style lang="scss">
.select-entity-dialog {
  .swap-tokens-list {
    display: flex;
    flex-direction: column;
    padding: 16px 0;
  }
  .swap-token {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    cursor: pointer;

    &:hover {
      background-color: color-mix(in oklab, $secondary 70%, transparent);
    }

    &--selected {
      background-color: color-mix(in oklab, $secondary 45%, transparent);
    }

    .asset-icon {
      width: 32px;
      height: 32px;
      object-fit: contain;
      border-radius: 50%;
    }

    .asset-data {
      min-width: 40px;
      flex: 1;

      &__top-row {
        display: flex;
        align-items: center;
        gap: 8px;
      }

      &__symbol {
        font-size: 14px;
        color: $text-primary;
      }

      &__badge {
        font-size: 10px;
        line-height: 1;
        color: $dark;
        padding: 4px 6px;
        border-radius: 999px;
        background-color: $brand-50;
      }

      &__name {
        font-size: $text-xs;
        color: $text-tertiary;
      }

      &__meta {
        font-size: 11px;
        color: $text-secondary;
      }
    }

    .asset-price {
      margin-left: auto;
      font-size: 12px;
      color: $text-secondary;
      white-space: nowrap;
    }
  }
}
</style>
