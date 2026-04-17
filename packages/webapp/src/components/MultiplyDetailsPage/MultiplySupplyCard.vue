<script lang="ts" setup>
import type { MultiplyVaultItem } from '~/types/table'

const { vault } = defineProps<{
  vault?: MultiplyVaultItem
}>()

const userStore = useUserStore()

const tabs = computed(() => {
  const dynamicTabs = [{ label: 'Open Position', value: 'supply' }]

  if (isUserHaveMultiply(vault)) {
    dynamicTabs.push({ label: 'Close Position', value: 'withdraw' })
  }
  return dynamicTabs
})

const activeTab = ref(tabs.value[0])

watchDebounced(tabs, (t) => {
  if (t.length === 1) {
    activeTab.value = t[0]
  }
}, { debounce: 300 })

function isUserHaveMultiply(vault?: MultiplyVaultItem) {
  if (!vault) { return false }
  return checkIsHaveMultiply(userStore.state.multiplyObligations, [vault] as any, vault.depositPoolData.pool.pool_address, vault.market)
}
</script>

<template>
  <div class="supply-card">
    <div class="supply-card__body">
      <div
        v-if="tabs.length > 1"
        class="supply-card-tabs mb-4"
      >
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

      <multiply-window
        v-if="activeTab?.value === 'supply'"
        :vault="vault"
      />
      <withdraw-multiply-window
        v-if="activeTab?.value === 'withdraw'"
        :vault="vault"
        opened
      />
    </div>
  </div>
</template>
