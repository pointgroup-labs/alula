<script lang="ts" setup>
const marketsStore = useMarketsStore()
const { getFullTokenData } = useTokensStore()

const activePool = computed(() => marketsStore.activeMarket?.marketState?.pools_data?.find(data => data.pool.pool_address === marketsStore.selectedPoolAddress))
const asset = computed(() => getFullTokenData(activePool.value?.pool?.token_symbol ?? ''))

const dialog = defineModel({ default: false })
const isOpen = ref(false)

watch(dialog, () => {
  setTimeout(() => {
    isOpen.value = dialog.value
  }, dialog.value ? 0 : 500)
})
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="pool-dialog dialog-default"
  >
    <template #header>
      <div class="dialog-default__title">
        <img
          :src="asset?.icon"
          :alt="`${asset?.symbol} icon`"
        >
        <span>Withdraw {{ asset?.symbol }}</span>
      </div>
    </template>

    <div
      v-if="isOpen"
      class="dialog-default__body"
    >
      <withdraw-window opened />
    </div>
  </j-dialog>
</template>
