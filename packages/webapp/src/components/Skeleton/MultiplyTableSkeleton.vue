<script lang="ts" setup>
const fields = [
  { key: 'asset', label: 'Pair', align: 'left' },
  { key: 'maxMultiplier', label: 'Multiplier', align: 'center' },
  { key: 'apyAtMaxMultiplier', label: 'Net APY', align: 'center' },
  { key: 'netEquity', label: 'Net Equity', align: 'right' },
  { key: 'action', label: '', align: 'right' },
]
const items = Array.from({ length: 3 }).map(() => {
  return {
    asset: '',
    maxMultiplier: '',
    apyAtMaxMultiplier: '',
    netEquity: '',
    action: '',
  }
})

const show = ref(true)

watch(show, (v) => {
  if (!v) { show.value = true }
})
</script>

<template>
  <b-accordion
    :flush="true"
    class="j-accordion table-skeleton"
    disabled
  >
    <b-accordion-item
      v-model="show"
      :visible="true"
    >
      <template #title>
        <j-skeleton
          style="border-radius: 40px;"
        />

        <div
          class="market-info-wrapper"
          style="margin-left: auto;"
        >
          <j-skeleton
            style="border-radius: 40px;"
            width="120"
          />
        </div>

      </template>
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

        <template #cell(maxMultiplier)>
          <div class="table-cell justify-content-center">
            <j-skeleton style="border-radius: 40px;" />
          </div>
        </template>

        <template #cell(apyAtMaxMultiplier)>
          <div class="table-cell justify-content-center">
            <j-skeleton style="border-radius: 40px;" />
          </div>
        </template>

        <template #cell(netEquity)>
          <div class="table-cell justify-content-end">
            <j-skeleton style="border-radius: 40px;" />
          </div>
        </template>

        <template #cell(action)>
          <div class="table-cell justify-content-end market-table__action">
            <j-skeleton style="border-radius: 40px;" />
          </div>
        </template>
      </btable></b-accordion-item>
  </b-accordion>
</template>
