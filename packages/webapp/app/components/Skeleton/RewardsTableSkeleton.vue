<script lang="ts" setup>
const fields = [
  { key: 'asset', label: 'Asset', align: 'left' },
  { key: 'market', label: 'Market', align: 'left' },
  { key: 'strategy', label: 'Strategy', align: 'left' },
  { key: 'pending', label: 'Pending', align: 'right' },
  { key: 'action', label: '', thClass: 'profile-action', tdClass: 'profile-action' },
]

const items = Array.from({ length: 1 }).map(() => {
  return {
    asset: '',
    market: '',
    strategy: '',
    pending: '',
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
    class="portfolio-table market-table"
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

    <template #cell(market)>
      <div class="d-flex justify-content-start">
        <j-skeleton
          width="60"
          height="20"
          pill
        />
      </div>
    </template>

    <template #cell(strategy)>
      <div class="d-flex justify-content-start">
        <j-skeleton
          width="60"
          height="20"
          pill
        />
      </div>
    </template>

    <template #cell(pending)>
      <div class="d-flex justify-content-end">
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
