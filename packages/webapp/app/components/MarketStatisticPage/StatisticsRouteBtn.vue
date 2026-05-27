<script lang="ts" setup>
const { marketName } = defineProps<{
  marketName: string
}>()

const marketsStore = useMarketsStore()
const router = useRouter()

function statistocRoute() {
  const marketAddress = marketsStore.state.markets[marketName]?.address
  router.push(`/statistics/${marketAddress}`)
}
</script>

<template>
  <market-info-badge
    class="statistics-badge"
    @click.stop="statistocRoute"
  >
    <span data-name="title">Statistics: </span>
    <i-app-statistics-icon style="width: 16px; height: 16px" />
  </market-info-badge>
</template>

<style lang="scss" scoped>
  .statistics-badge {
  border-color: rgba($cyan, 0.4);
  &:hover {
    border-color: rgba($cyan, 1);
    color: $cyan;
    cursor: pointer;

    span {
      color: $cyan;
    }
  }
}
</style>
