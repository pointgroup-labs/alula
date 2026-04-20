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
  label: 'Position',
  value: 'position',
}]

const defaultTab = tabs[0]!

function resolveTab(tabValue?: string | null) {
  return tabs.find(tab => tab.value === tabValue) ?? defaultTab
}

const activeTab = ref(resolveTab(route.params.page as string | undefined))

watch(() => activeTab.value?.value, (val) => {
  if (!val) {
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
  <main class="multiply-details container">
    <multiply-details-top />

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
      />

      <multiply-details-overview
        v-if="activeTab.value === 'pool'"
        :selected-vault="selectedVault"
      />
      <leverage-position v-if="activeTab.value === 'position'" />
    </template>

    <div
      v-else
      class="multiply-details__empty"
    >
      Market or pool not found.
    </div>
  </main>
</template>

<style lang="scss">
.multiply-details {
  display: flex;
  flex-direction: column;
  gap: 32px;
  padding-bottom: 72px;

  &__eyebrow {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: $text-brand;
  }

  &__empty {
    padding: 32px;
    border-radius: 24px;
    background: $bg-card;
    border: 1px solid $border-primary;
    color: $text-secondary;
    text-align: center;
  }
}
</style>
