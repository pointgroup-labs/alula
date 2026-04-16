<script lang="ts" setup>
const fields = [
  { key: 'asset', label: 'Vault', align: 'left' },
  { key: 'market', label: 'Market', align: 'center' },
  { key: 'liquidity', label: 'Borrow Liquidity', align: 'right' },
  { key: 'supplied', label: 'Collateral TVL', align: 'right' },
  { key: 'apyAtMaxMultiplier', label: 'APY at Max Multiplier', align: 'center' },
  { key: 'maxMultiplier', label: 'Max Multiplier', align: 'center' },
  { key: 'action', label: '', align: 'right' },
]
const items = Array.from({ length: 3 }).map(() => {
  return {
    asset: '',
    market: '',
    liquidity: '',
    supplied: '',
    apyAtMaxMultiplier: '',
    maxMultiplier: '',
    action: '',
  }
})

const show = ref(true)

watch(show, (v) => {
  if (!v) { show.value = true }
})
</script>

<template>
  <BTable
    show-empty
    borderless
    :fields="fields"
    :items="items"
    responsive
    class="market-table multiply-table__desktop"
  >
    <template
      v-for="field in fields"
      :key="field.key"
      #[`head(${field.key})`]="data"
    >
      <span :style="{ '--align': field.align }">{{ data.label }}</span>
    </template>

    <template #cell(asset)>
      <div class="market-table__asset">
        <j-skeleton
          width="32"
          height="32"
          variant="rounded"
        />
        <j-skeleton
          width="32"
          height="32"
          variant="rounded"
        />
        <div class="market-table__asset__info">
          <j-skeleton
            style="border-radius: 40px;"
            width="130"
          />
        </div>
      </div>
    </template>

    <template #cell(market)>
      <div class="table-cell justify-content-center market-cell">
        <j-skeleton style="border-radius: 40px;" />
      </div>
    </template>

    <template #cell(liquidity)>
      <div class="table-cell justify-content-end">
        <j-skeleton style="border-radius: 40px;" />
      </div>
    </template>

    <template #cell(supplied)>
      <div class="table-cell justify-content-end">
        <j-skeleton style="border-radius: 40px;" />
      </div>
    </template>

    <template #cell(apyAtMaxMultiplier)>
      <div class="table-cell justify-content-center">
        <j-skeleton style="border-radius: 40px;" />
      </div>
    </template>

    <template #cell(maxMultiplier)>
      <div class="table-cell justify-content-center">
        <j-skeleton style="border-radius: 40px;" />
      </div>
    </template>

    <template #cell(action)>
      <div class="table-cell justify-content-end market-table__action">
        <j-skeleton style="border-radius: 40px;" />
      </div>
    </template>
  </BTable>
</template>
