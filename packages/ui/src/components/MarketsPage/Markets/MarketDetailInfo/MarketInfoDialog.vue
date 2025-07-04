<script lang="ts" setup>
const {
  modelValue,
} = defineProps<{
  modelValue: boolean
}>()

const emits = defineEmits(['update:modelValue'])

const marketsStore = useMarketsStore()
const market = computed(() => marketsStore.selectedMarketInfo)

const dialog = computed({
  get() {
    return modelValue
  },
  set(val) {
    emits('update:modelValue', val)
  },
})

watch(market, (val) => {
  console.log(val)
})
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="market-info-dialog"
  >
    <template #header>
      <div class="market-info">
        <img :src="market?.asset.icon" :alt="market?.asset.symbol">
        {{ market?.asset.name }} Info
        <j-pill-label variant="secondary" size="md" bg-color="#08b57680">
          Can by collateral
        </j-pill-label>
      </div>
    </template>

    <div class="market-info__body">
      <market-details-supply />
      <div class="separator-vert" />
      <market-details-borrow />
    </div>

    <div class="separator" />

    <market-details-bottom />
  </j-dialog>
</template>

<style lang="scss">
.market-info-dialog {
  .modal-content {
    max-width: 1104px;
  }

  .modal-body {
    display: flex;
    flex-direction: column;
    gap: $spacing-16;
    overflow: initial;
  }

  .market-info__body {
    display: flex;
    gap: $spacing-24;
    padding-top: $spacing-16;
  }

  .market-info {
    font-size: 20px;
    font-style: normal;
    font-weight: 500;
    line-height: 20px;
    display: flex;
    align-items: center;
    gap: $spacing-8;

    img {
      width: 40px;
      height: 40px;
      object-fit: contain;
      border-radius: 50%;
    }

    .j-pill-label {
      margin-left: 2px;
      font-size: 11px;
      font-style: normal;
      font-weight: 500;
      line-height: 12px;
    }
  }
}
</style>
