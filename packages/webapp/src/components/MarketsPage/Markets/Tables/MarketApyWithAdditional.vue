<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'

const {
  poolData,
  additionalMarketsData,
  isDeposit = true,
} = defineProps<{
  poolData?: MarketTableItem
  additionalMarketsData: any
  isDeposit?: boolean
}>()

const additionalData = computed(() => {
  if (!poolData) {
    return null
  }
  const market = additionalMarketsData.find((item: any) => item.marketName === poolData.market)
  const data = market?.data?.[poolData.pool_address]
  if (!data) {
    return null
  }
  if (Object.keys(data).length === 0) {
    return null
  }
  return data.find((item: any) => item.name === (isDeposit ? 'deposit' : 'borrow'))
})

const additionalApy = computed(() => additionalData.value?.additional_apy ?? 0)

const poolApy = computed(() => isDeposit ? poolData?.deposit_apy : poolData?.borrow_apy)

const apyWithAdittional = computed(() => {
  if (!additionalData.value) {
    return 0
  }
  const poolApyNum = Number.parseFloat(poolApy.value ?? '0')
  const totalApy = isDeposit ? poolApyNum + additionalApy.value : Math.max(poolApyNum - additionalApy.value, 0)
  return totalApy || 0
})
</script>

<template>
  <div class="table-cell justify-content-center flex">
    <lighting-apy
      v-if="additionalData"
      :label="`${truncatePercent(apyWithAdittional, 2)}%`"
      :variant="isDeposit ? 'deposit' : 'borrow'"
    >
      <template #tip>
        <div class="additional-tip">
          <div class="additional-tip__title">
            This position earns additional incentives:
          </div>
          <div class="separator" />
          <div class="additional-tip__value">
            <span>Additional APY: </span> {{ truncatePercent(additionalApy, 2) }}%
          </div>
          <div class="additional-tip__value">
            <span>{{ isDeposit ? 'Deposit' : 'Borrow' }} APY: </span> {{ poolApy }}
          </div>
          <div class="additional-tip__value">
            <span>Total APY: </span> {{ truncatePercent(apyWithAdittional, 2) }}%
          </div>
        </div>
      </template>
    </lighting-apy>
    <j-pill-label
      v-else
      color="#111"
      :variant="isDeposit ? 'success' : 'warning'"
      size="sm"
    >
      {{ poolApy }}
    </j-pill-label>

  </div>
</template>

<style lang="scss">
.additional-tip {
  display: flex;
  flex-direction: column;
  gap: 4px;

  .separator {
    margin: 4px 0 8px;
  }

  &__title {
    font-weight: 500;
    font-size: 12px;
  }

  &__value {
    width: 100%;
    display: flex;
    font-weight: 600;
    font-size: 14px;
    justify-content: space-between;
    padding: 2px 0;

    span {
      font-weight: 500;
      font-size: 12px;
    }
  }
}
</style>
