<script lang="ts" setup>
const route = useRoute()
const router = useRouter()

const marketsStore = useMarketsStore()
const loading = computed(() => marketsStore.state.loading)

const multiplyStore = useMultiplyStore()
const selectedVault = computed(() => multiplyStore.selectedVault)

const tabs = [{
  label: 'Pool',
  value: 'pool',
},
{
  label: 'Info / Risks',
  value: 'info',
},
{
  label: 'Position',
  value: 'position',
},
{
  label: 'Statistics',
  value: 'statistics',
}]

const defaultTab = tabs[0]!

function resolveTab(tabValue?: string | null) {
  return tabs.find(tab => tab.value === tabValue) ?? defaultTab
}

const activeTab = ref(resolveTab(route.params.page as string | undefined))

function statisticsRoute() {
  router.push(`/statistics/${selectedVault.value?.marketAddress}/${selectedVault.value?.pairKey}`)
}

watch(() => activeTab.value?.value, (val) => {
  if (!val || val === 'statistics') {
    return
  }

  if (route.params.page === val) {
    return
  }

  router.push({
    params: {
      ...route.params,
      page: val,
    },
  })
})
</script>

<template>
  <j-loading-spinner
    v-if="loading"
    class="table-loading-spinner"
  >
    Loading market data...
  </j-loading-spinner>

  <template v-else-if="selectedVault">
    <j-line-tab
      v-model="activeTab"
      :tabs="tabs"
      style="margin-bottom: -12px;"
    >
      <template #tab="{ tab }">
        <span
          v-if="tab.value === 'statistics'"
          class="tab-statistics tab-label"
          @click.stop="statisticsRoute"
        >
          <i-app-statistics-icon />  Statistics
        </span>
        <span
          v-else
          class="tab-label"
        >
          {{ tab.label }}
        </span>

      </template>
    </j-line-tab>

    <multiply-details-overview
      v-if="activeTab.value === 'pool'"
      :selected-vault="selectedVault"
    />
    <multiply-info-risks
      v-if="activeTab.value === 'info'"
    />
    <leverage-position v-if="activeTab.value === 'position'" />
  </template>

  <div
    v-else
    class="multiply-details__empty"
  >
    Market or pool not found.
  </div>
</template>

<style lang="scss">
.multiply-details {
  display: flex;
  flex-direction: column;
  gap: 32px;
  padding-bottom: 72px;

  .loading-spinner,
  .multiply-details__empty {
    padding: 32px;
    margin: 0 auto;
    color: $navi-50;
    font-size: 12px;
    font-style: normal;
    font-weight: 400;
    line-height: 16px;
    gap: 12px;
  }

  .j-line-tabs {
    .overview-tab:has(.tab-statistics) {
      padding: 0;
    }
    .tab-statistics {
      padding: 0 6px 9px;
      text-decoration: none;
      color: #fff;
      font-weight: 500;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 8px;
      margin-bottom: -1px;

      svg {
        width: 18px;
        height: 18px;
      }
    }
  }
}
</style>
