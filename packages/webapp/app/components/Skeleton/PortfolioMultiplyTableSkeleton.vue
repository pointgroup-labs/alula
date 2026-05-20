<script lang="ts" setup>
const fields = [
  { key: 'asset', label: 'Vault', align: 'left' },
  { key: 'multiplier', label: 'Multiplier', align: 'center' },
  { key: 'market', label: 'Market', align: 'center' },
  { key: 'deposited', label: 'Deposited', align: 'right' },
  { key: 'borrowed', label: 'Borrowed', align: 'right' },
  { key: 'hf', label: 'Health Factor', align: 'right' },
  { key: 'action', label: '' },
]
const items = Array.from({ length: 3 }).map(() => {
  return {
    asset: '',
    multiplier: '',
    market: '',
    deposited: '',
    borrowed: '',
    hf: '',
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
          />
        </div>
      </div>
    </template>

    <template #cell(multiplier)>
      <div class="table-cell justify-content-center market-cell">
        <j-skeleton style="border-radius: 40px;" />
      </div>
    </template>

    <template #cell(market)>
      <div class="table-cell justify-content-center market-cell">
        <j-skeleton style="border-radius: 40px;" />
      </div>
    </template>

    <template #cell(deposited)>
      <div class="table-cell justify-content-end">
        <j-skeleton style="border-radius: 40px;" />
      </div>
    </template>

    <template #cell(borrowed)>
      <div class="table-cell justify-content-end">
        <j-skeleton style="border-radius: 40px;" />
      </div>
    </template>

    <template #cell(hf)>
      <div class="table-cell justify-content-end">
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
