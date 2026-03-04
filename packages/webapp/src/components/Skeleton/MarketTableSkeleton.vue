<script lang="ts" setup>
const fields = [
  { key: 'asset', label: 'Asset', align: 'left', thClass: 'asset', tdClass: 'asset' },
  { key: 'total_supply', label: 'Supplied', align: 'right', thClass: 'supply', tdClass: 'supply' },
  { key: 'total_borrowed', label: 'Borrowed', align: 'right', thClass: 'borrow', tdClass: 'borrow' },
  { key: 'utilization_rate', label: 'Utilization', align: 'right', thClass: 'utilization', tdClass: 'utilization' },
  { key: 'deposit_apy', label: 'Supply APY', align: 'center', thClass: 'apy', tdClass: 'apy' },
  { key: 'borrow_apy', label: 'Borrow rate', align: 'center', thClass: 'apy', tdClass: 'apy' },
  { key: 'position', label: 'My Position', align: 'right', thClass: 'position', tdClass: 'position' },
  { key: 'action', label: '', thClass: 'action', tdClass: 'action' },
]
const items = Array.from({ length: 3 }).map(() => {
  return {
    asset: '',
    total_supply: '',
    total_borrowed: '',
    utilization_rate: '',
    deposit_apy: '',
    borrow_apy: '',
    position: '',
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
          pill
          width="60"
          height="30"
        />

        <j-skeleton
          pill
          width="145"
          height="30"
          style="margin: 0 20px 0 auto"
        />

        <j-skeleton
          pill
          width="145"
          height="30"
        />
      </template>

      <BTable
        show-empty
        borderless
        :fields="fields"
        :items="items"
        responsive
        class="market-table market-skeleton-table"
      >
        <template
          v-for="field in fields"
          :key="field.key"
          #[`head(${field.key})`]="data"
        >
          <span :style="{ '--align': field.align }">{{ data.label }}</span>
        </template>

        <template #cell(asset)>
          <div class="d-flex align-items-center">
            <j-skeleton
              width="32"
              height="32"
              variant="rounded"
            />
            <j-skeleton
              width="52"
              height="20"
              class="mx-2"
              pill
            />
          </div>
        </template>

        <template #cell(total_supply)>
          <div class="d-flex justify-content-end">
            <j-skeleton
              width="60"
              height="20"
              pill
            />
          </div>
        </template>

        <template #cell(total_borrowed)>
          <div class="d-flex justify-content-end">
            <j-skeleton
              width="60"
              height="20"
              pill
            />
          </div>
        </template>

        <template #cell(utilization_rate)>
          <div class="d-flex justify-content-end">
            <j-skeleton
              width="50"
              height="20"
              pill
            />
          </div>
        </template>

        <template #cell(deposit_apy)>
          <div class="d-flex justify-content-center">
            <j-skeleton
              width="56"
              height="20"
              pill
            />
          </div>
        </template>

        <template #cell(borrow_apy)>
          <div class="d-flex justify-content-center">
            <j-skeleton
              width="56"
              height="20"
              pill
            />
          </div>
        </template>

        <template #cell(position)>
          <div class="d-flex justify-content-end">
            <j-skeleton
              width="56"
              height="20"
              pill
            />
          </div>
        </template>

        <template #cell(action)>
          <div class="d-flex justify-content-end">
            <j-skeleton
              width="70"
              height="20"
              pill
              style="margin-right: 8px;"
            />

            <j-skeleton
              width="70"
              height="20"
              pill
            />
          </div>
        </template>
      </BTable>
    </b-accordion-item>
  </b-accordion>
</template>

<style lang="scss">
.table-skeleton {
  pointer-events: none;
  .accordion-body {
    padding: 4px 0 0 !important;
  }
}
.market-skeleton-table {
  tbody {
    tr {
      height: 65px;
    }
  }
  .asset {
    padding-left: 32px !important;
  }
}
</style>
