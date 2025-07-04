<script lang="ts" setup>
const tabs = ['Supply', 'Borrow']
const activeTab = ref('Supply')
const infoDialog = ref(false)
</script>

<template>
  <div class="markets card">
    <j-btn-group v-model="activeTab" :buttons="tabs" class="markets-tabs">
      <template #default="{ select, isActive, label }">
        <j-btn :variant="isActive ? activeTab === tabs[0] ? 'primary' : 'accent' : 'secondary'" size="lg" @click="select">
          {{ label }}
          <i-app-strongbox-icon v-if="label === tabs[0]" />
          <i-app-percentage-square-icon v-if="label === tabs[1]" />
        </j-btn>
      </template>
    </j-btn-group>

    <keep-alive>
      <supply-table v-if="activeTab === tabs[0]" @show-info="infoDialog = true" />
    </keep-alive>
    <keep-alive>
      <borrow-table v-if="activeTab === tabs[1]" @show-info="infoDialog = true" />
    </keep-alive>

    <market-info-dialog v-model="infoDialog" />
  </div>
</template>

<style lang="scss">
.markets {
  display: flex;
  flex-direction: column;
  gap: $spacing-24;

  .markets-tabs {
    width: fit-content;

    .btn {
      min-width: 200px;

      &-content {
        gap: $spacing-4;
      }
    }

    svg {
      width: 24px;
      height: 24px;
    }
  }
}
</style>
