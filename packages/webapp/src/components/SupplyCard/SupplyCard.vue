<script lang="ts" setup>
const tabs = [
  { label: 'Supply', value: 'supply' },
  { label: 'Borrow', value: 'borrow' },
]

const activeTab = ref(tabs[0])

const dialog = ref(false)

function dialogHandler() {
  dialog.value = !dialog.value
}
</script>

<template>
  <div class="supply-card">
    <div class="supply-card__body">
      <div class="supply-card-tabs">
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

      <change-pool-dialog v-model="dialog" />
    </div>
  </div>
</template>