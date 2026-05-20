<script lang="ts" setup>
const fields = [
  { key: 'asset', label: 'Asset', align: 'left' },
  { key: 'balance', label: 'Supply', align: 'right' },
  { key: 'supply_apy', label: 'Supply APY', align: 'center' },
  { key: 'action', label: '', thClass: 'profile-action', tdClass: 'profile-action' },
]
const items = Array.from({ length: 1 }).map(() => {
  return {
    asset: '',
    balance: '',
    supply_apy: '',
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
    class="account-table market-table account-skeleton-table"
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

    <template #cell(balance)>
      <div class="d-flex justify-content-end">
        <j-skeleton
          width="60"
          height="20"
          pill
        />
      </div>
    </template>

    <template #cell(supply_apy)>
      <div class="d-flex justify-content-center">
        <j-skeleton
          width="60"
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
        />
      </div>
    </template>
  </BTable>
</template>

<style lang="scss">
.market-skeleton-table {
  pointer-events: none;
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
