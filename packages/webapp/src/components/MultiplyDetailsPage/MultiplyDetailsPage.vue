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
    <back-btn to="/multiply" />

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

      <multiply-details-main
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

  &__hero {
    display: grid;
    grid-template-columns: minmax(0, 1.4fr) minmax(320px, 0.9fr);
    gap: 20px;
    padding: 32px;
    border-radius: 32px;
    background:
      radial-gradient(circle at top left, rgba(34, 211, 238, 0.12), transparent 34%),
      linear-gradient(180deg, rgba(17, 24, 39, 0.96) 0%, rgba(13, 18, 31, 0.96) 100%);
    border: 1px solid $border-primary;
    box-shadow: 0 18px 48px rgba(0, 0, 0, 0.24);

    @media (max-width: $breakpoint-lg) {
      grid-template-columns: 1fr;
    }
  }

  &__eyebrow {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: $text-brand;
  }

  &__title {
    margin: 10px 0;
    font-size: clamp(28px, 4vw, 44px);
    line-height: 1;
    color: $text-primary;
  }

  &__copy {
    margin: 0;
    max-width: 720px;
    line-height: 1.7;
    color: $text-tertiary;
  }

  &__hero-stats,
  &__overview {
    height: fit-content;
    display: grid;
    gap: 14px;
    grid-template-columns: repeat(2, minmax(0, 1fr));

    @media (max-width: $breakpoint-sm) {
      grid-template-columns: 1fr;
    }
  }

  &__hero-stats > div,
  &__card {
    height: fit-content;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 16px;
    border-radius: 18px;
    background: $surface-neutral-04;
    border: 1px solid $surface-neutral-08;

    span,
    small {
      color: $text-tertiary;
      font-size: 12px;
    }

    strong {
      font-size: 15px;
      color: $text-primary;
      word-break: break-all;
      display: flex;
      align-items: center;
      gap: 12px;
    }
  }

  &__content {
    display: flex;
    justify-content: space-between;
    gap: 20px;

    @media (max-width: $breakpoint-xs) {
      flex-direction: column;
    }
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
