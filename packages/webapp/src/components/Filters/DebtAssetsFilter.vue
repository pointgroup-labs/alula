<script lang="ts" setup>
import type { ComponentPublicInstance } from 'vue'

const { scope = 'markets' } = defineProps<{ scope?: FilterScope }>()

const filterStore = useMarketFilterStore()

const uniqueAssets = computed(() => filterStore.uniqueAssets)

const el = ref<ComponentPublicInstance | null>(null)

const menuWidth = computed(() => (el.value?.$el as HTMLElement | undefined)?.offsetWidth ?? 100)

function toggle(symbol: string) {
  filterStore.toggle(scope, 'debt', symbol)
}
</script>

<template>
  <j-popover
    v-if="uniqueAssets.length > 0"
    variant="ghost"
    position="bottom"
    :teleport-to-body="false"
  >
    <div
      class="markets-filter-menu"
      :style="{ width: `${menuWidth - 24}px` }"
    >
      <div
        v-for="asset in uniqueAssets"
        :key="asset.symbol"
        class="filter-item"
        @click="toggle(asset.symbol)"
      >
        <j-checkbox
          v-model="filterStore.filters[scope].debt[asset.symbol]"
          size="md"
          @click.stop
        />
        <img
          :src="asset.icon"
          alt="token icon"
        >
        <span>{{ asset.symbol }}</span>
      </div>
    </div>
    <template #target="{ active }">
      <j-btn
        ref="el"
        :variant="filterStore.isActiveDebtFilter(scope) ? 'brand-outlined' : 'ghost'"
        size="md"
        class="market-filter-btn"
      >
        Debt Assets <i-app-accordion-arrow-down
          class="arrow-icon"
          :class="{ active }"
        />
      </j-btn>
    </template>
  </j-popover>
</template>
