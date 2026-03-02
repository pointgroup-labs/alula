/**
* APY display component with optional additional incentives.
*
* - If `additionalMarketsData` contains incentive data for the current pool,
* the component shows a highlighted APY with a lightning icon.
* This value represents the **total APY**, including base pool APY
* and additional incentives.
*/
<script lang="ts" setup>
import type { MarketTableItem } from '~/types/table'

const {
  poolData,
  additionalMarketsData,
  isDeposit = true,
} = defineProps<{
  poolData?: MarketTableItem
  additionalMarketsData?: any
  isDeposit?: boolean
}>()

const additionalData = computed(() => {
  if (!poolData) {
    return null
  }
  const market = additionalMarketsData?.find((item: any) => item.marketName === poolData.market)
  const data = market?.data?.[poolData.pool_address]
  if (!data) {
    return null
  }
  if (Object.keys(data).length === 0) {
    return null
  }
  return data.filter((item: any) => item.name === (isDeposit ? 'deposit' : 'borrow'))
})

const poolApy = computed(() => isDeposit ? poolData?.deposit_apy : poolData?.borrow_apy)

const apyWithAdittional = computed(() => {
  if (!additionalData.value || additionalData.value.length === 0) {
    return 0
  }
  const poolApyNum = Number.parseFloat(poolApy.value ?? '0')
  const additionalApy = additionalData.value.reduce((acc: typeof additionalData.value, item: typeof additionalData.value[number]) => acc + item.additional_apy, 0)
  const totalApy = isDeposit ? poolApyNum + additionalApy : Math.max(poolApyNum - additionalApy, 0)
  return totalApy || 0
})
</script>

<template>
  <lighting-apy
    v-if="additionalData && additionalData.length > 0"
    :label="`${truncatePercent(apyWithAdittional, 2)}%`"
    :variant="isDeposit ? 'deposit' : 'borrow'"
  >
    <template #tip>
      <div class="additional-tip">
        <div class="additional-tip__title">
          This position earns additional incentives:
        </div>
        <div class="separator" />
        <div
          v-for="data in additionalData"
          :key="data.additional_apy"
          class="additional-tip__value"
        >
          {{ data.token_symbol }} REWARDS: <span :style="{ color: isDeposit ? '#22d3ee' : '#6366F1' }">
            {{ truncatePercent(data.additional_apy, 2) }}% </span>
        </div>
        <div class="additional-tip__value">
          {{ isDeposit ? 'Lending' : 'Borrow' }} APY:<span>{{ poolApy }}</span>
        </div>

        <div class="separator" />

        <div class="additional-tip__value">
          Total Combined APY: <span>{{ truncatePercent(apyWithAdittional, 2) }}% </span>
        </div>
      </div>
    </template>
  </lighting-apy>
  <j-pill-label
    v-else
    :variant="isDeposit ? 'supply' : 'borrow'"
    size="sm"
  >
    {{ poolApy }}
  </j-pill-label>
</template>

<style lang="scss">
.additional-tip {
  display: flex;
  flex-direction: column;
  gap: 4px;

  .separator {
    margin: 12px 0;
    background-color: $surface-neutral-10;
  }

  &__title {
    font-weight: 700;
    font-size: 12px;
  }

  &__value {
    width: 100%;
    display: flex;
    justify-content: space-between;
    padding: 2px 0;
    font-weight: 400;
    font-size: 12px;

    span {
      font-weight: 700;
      font-size: 14px;
    }
  }
}
</style>
