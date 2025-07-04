<script lang="ts" setup>
const {
  data,
} = defineProps<{
  data: any[]
}>()

function formatChartLabels(data: any[]) {
  const labels: { label: string, value: number | string, pixelSpacing: number }[] = []
  for (const item of data) {
    const [label, value] = item?.label?.split(' ')
    labels.push({
      label,
      value,
      pixelSpacing: item.pixelSpacing || 0,
    })
  }

  return labels
}

const chartDataLabels = computed(() => {
  return formatChartLabels(data)
})

const labelEl = ref()

const labelsHeight = computed(() => {
  const labels = labelEl.value ?? []
  return `${Math.max(...labels.map((l: any) => l?.getBoundingClientRect().height), 30)}px`
})

function getLabelWidth(idx: number) {
  if (!labelEl.value) {
    return 42 / 2
  }
  return labelEl.value[idx]?.getBoundingClientRect()?.width / 2 || 42 / 2
}
</script>

<template>
  <div class="chart-legend" :style="{ height: labelsHeight }">
    <div
      v-for="(d, idx) in chartDataLabels"
      :key="`${d.label}-${idx}`"
      ref="labelEl"
      :style="{ left: idx === 0 ? `${d.pixelSpacing - 20}px` : `${d.pixelSpacing - getLabelWidth(idx)}px` }"
      class="chart-legend__label"
    >
      <span :data-name="!d.value ? 'label' : 'title'">{{ d.label }} </span>
      <span v-if="d.value">{{ d.value }}</span>
    </div>
  </div>
</template>

<style lang="scss">
  .chart-legend {
  position: relative;
  display: flex;
  justify-content: flex-start;
  z-index: 0;

  &__label {
    position: absolute;
    z-index: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: $spacing-4;
    color: $neutral-6;
    text-align: center;
    font-style: normal;
    font-weight: 500;
    line-height: 8px;
    border-radius: $spacing-4;
    background: $neutral-5;
    padding: $spacing-4;
    min-width: 42px;
    width: 42px;
    height: 30px;

    @media (max-width: $breakpoint-xs) {
      flex-direction: row;
      justify-content: flex-end;
      align-items: center;
      writing-mode: vertical-lr;
      text-orientation: mixed;
      white-space: nowrap;
      transform: rotate(180deg);
      height: auto;
      background: none;
      padding: 6px 0;

      &::before {
        content: '';
        width: 16px;
        height: 100%;
        position: absolute;
        top: 0;
        left: 50%;
        transform: translateX(-50%);
        background: $neutral-5;
        z-index: -1;
      }

      span {
        font-size: 8px !important;
      }
    }

    span[data-name='title'] {
      font-size: 8px;
    }

    span[data-name='label'] {
      font-size: 11px;
    }

    span:nth-child(2) {
      font-size: 11px;
    }
  }
}

body.body--dark {
  .chart-legend__label {
    background-color: $dark;
    color: $neutral-3;

    @media (max-width: $breakpoint-xs) {
      background-color: transparent;

      &::before {
        background-color: $dark;
      }
    }
  }
}
</style>
