<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'

const selectedPool = inject<Ref<MarketTableItem>>('selectedPool')

const route = useRoute()
const router = useRouter()

const supplied = computed(() => Number(selectedPool?.value.position.supplied) || 0)
const borrowed = computed(() => Number(selectedPool?.value.position.borrowed) || 0)

const tabs = computed(() => {
  if (supplied.value > 0) {
    return [
      { label: 'Supply', value: 'supply' },
      { label: 'Withdraw', value: 'withdraw' },
    ]
  }
  if (borrowed.value > 0) {
    return [
      { label: 'Borrow', value: 'borrow' },
      { label: 'Repay', value: 'repay' },
    ]
  }
  return [
    { label: 'Supply', value: 'supply' },
    { label: 'Borrow', value: 'borrow' },
  ]
})

const activeTab = ref(tabs.value[0])

const dialog = ref(false)

function dialogHandler() {
  dialog.value = !dialog.value
}

watchDebounced(tabs, (t) => {
  activeTab.value = t[0]
}, { debounce: 300 })

watch(activeTab, () => {
  if (route.query?.action) {
    const query = { ...route.query }
    delete query?.action
    router.replace({
      name: route.name as string,
      params: {
        ...route.params,
      },
      query,
    })
  }
})

watch(() => route.query, (query) => {
  const action = query?.action
  if (!action) {
    return
  }
  const findTab = tabs.value.find(t => t.value === action)
  if (findTab) {
    activeTab.value = findTab
    requestAnimationFrame(() => {
      focusInput('.borrow-input-wrapper')
    })
  }
}, { immediate: true })
</script>

<template>
  <div class="supply-card">
    <div class="supply-card__body">
      <div class="supply-card-tabs mb-4">
        <div
          v-for="tab in tabs"
          :key="tab.value"
          class="nav-tab"
          :class="[`nav-tab--${tab.value}`, { active: tab.value === activeTab?.value }]"
          @click="activeTab = tab"
        >
          {{ tab.label }}
        </div>
      </div>

      <supply-window
        v-if="activeTab?.value === 'supply'"
        @dialog-handler="dialogHandler"
      />
      <borrow-window
        v-else-if="activeTab?.value === 'borrow'"
        @dialog-handler="dialogHandler"
      />
      <repay-window
        v-else-if="activeTab?.value === 'repay'"
      />
      <withdraw-window
        v-else-if="activeTab?.value === 'withdraw'"
      />

      <change-pool-dialog v-model="dialog" />
    </div>
  </div>
</template>
