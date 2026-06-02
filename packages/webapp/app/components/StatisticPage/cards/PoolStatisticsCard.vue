<script lang="ts" setup>
import type { ECharts, EChartsOption, LineSeriesOption } from 'echarts'
import type { ApiHistoryData } from '~/services'
import * as echarts from 'echarts'

const {
  chartType,
  onlyMarketAsset,
  onlyPairAsset,
} = defineProps<{
  chartType: keyof ApiHistoryData
  onlyMarketAsset?: boolean
  onlyPairAsset?: boolean
}>()

const {
  cardLabel,
  cardValue,
  activeFilter,
  chartFilter,
  chartPoints,
  symbol,
  pairChartPoints,
  pairSymbol,
  hasPairPool,
  currencyOptions,
  currency,
} = usePoolStatistics({ chartType, onlyPairAsset })

const statisticsStore = useMarketStatisticsStore()

const isLoading = computed(() => statisticsStore.state.loading)

const isShowSelect = computed(() => ['total_supplied', 'total_borrowed'].includes(chartType))

const isUsdValue = computed(() => currency.value === 'USD')

// ---- ECharts ----
const el = ref<HTMLElement | null>(null)
let chart: ECharts | null = null

const isPercent = computed(() =>
  chartType === 'supply_apy_bps'
  || chartType === 'borrow_apy_bps'
  || chartType === 'utilization_bps',
)

const seriesColor = computed(() => {
  switch (chartType) {
    case 'total_borrowed':
    case 'borrow_apy_bps': return '#8a8df4'
    case 'utilization_bps': return '#f59e0b'
    case 'tvl_usd_cents': return '#1dc978'
    default: return '#22d3ee'
  }
})

const pairSeriesColor = computed(() => {
  switch (chartType) {
    case 'total_borrowed':
    case 'borrow_apy_bps': return '#f472b6'
    case 'utilization_bps': return '#fb7185'
    case 'tvl_usd_cents': return '#bed334'
    default: return '#60a5fa'
  }
})

const areaColorMap: Record<string, [string, string]> = {
  '#22d3ee': ['rgba(34,211,238,0.35)', 'rgba(34,211,238,0)'],
  '#8a8df4': ['rgba(138,141,244,0.35)', 'rgba(138,141,244,0)'],
  '#f59e0b': ['rgba(245,158,11,0.35)', 'rgba(245,158,11,0)'],
  '#1dc978': ['rgba(29, 201, 121, 0.35)', 'rgba(29, 201, 121, 0)'],
}

function formatAxis(v: number) {
  if (isPercent.value) { return `${truncatePercent(v, 2)}%` }
  if (isShowSelect.value && !isUsdValue.value) { return shortenNumber(v, 0, 0) }
  return `$${shortenNumber(v, 0, 0)}`
}

function formatTooltip(v: number) {
  if (isPercent.value) { return `${truncatePercent(v, 2)}%` }
  if (isShowSelect.value && !isUsdValue.value) { return `${formatPrice(v, 5, 5)}` }
  return `$${formatPrice(v, 2, 2)}`
}

const option = computed<EChartsOption>(() => {
  const color = seriesColor.value
  const pairColor = pairSeriesColor.value
  const gridLine = 'rgba(120, 160, 200, 0.18)'
  const axisText = 'rgba(180, 200, 220, 0.55)'
  const labels = chartPoints.value.map(p => p.label)
  const values = chartPoints.value.map(p => p.value)
  const pairValues = pairChartPoints.value.map(p => p.value)
  const series: LineSeriesOption[] = []
  if (!onlyPairAsset) {
    series.push({
      name: symbol.value || 'Pool',
      type: 'line',
      data: values,
      smooth: 0.45,
      showSymbol: false,
      lineStyle: { width: 2, color, cap: 'round', join: 'round' },
      itemStyle: { color },
      areaStyle: {
        opacity: 1,
        color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
          { offset: 0, color: (areaColorMap[color] ?? areaColorMap['#22d3ee']!)[0] },
          { offset: 1, color: (areaColorMap[color] ?? areaColorMap['#22d3ee']!)[1] },
        ]),
      },
    })
  }

  if (hasPairPool.value && pairValues.length > 0 && !onlyMarketAsset) {
    let pairSeria: LineSeriesOption = {
      name: pairSymbol.value || 'Pair',
      type: 'line',
      data: pairValues,
      smooth: 0.45,
      showSymbol: false,
    }
    pairSeria = onlyPairAsset
      ? {
          ...pairSeria,
          lineStyle: { width: 2, color, cap: 'round', join: 'round' },
          itemStyle: { color },
          areaStyle: {
            opacity: 1,
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: (areaColorMap[color] ?? areaColorMap['#22d3ee']!)[0] },
              { offset: 1, color: (areaColorMap[color] ?? areaColorMap['#22d3ee']!)[1] },
            ]),
          },
        }
      : {
          ...pairSeria,
          smooth: 0.45,
          showSymbol: false,
          lineStyle: { width: 2, color: pairColor, cap: 'round', join: 'round', type: 'dashed' },
          itemStyle: { color: pairColor },
          areaStyle: { opacity: 0 },
        }
    series.push(pairSeria)
  }

  return {
    backgroundColor: 'transparent',
    animation: true,
    animationDuration: 500,
    animationDurationUpdate: 400,
    animationEasingUpdate: 'cubicOut',

    grid: { left: 0, right: 15, top: 6, bottom: 20, containLabel: true },

    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'line', lineStyle: { color: gridLine, type: 'dashed' } },
      padding: 10,
      backgroundColor: 'rgba(10,14,23,0.92)',
      borderColor: 'rgba(255,255,255,0.06)',
      borderWidth: 1,
      textStyle: { color: '#E8EDF5', fontSize: 12 },
      formatter: (params: any) => {
        const arr = Array.isArray(params) ? params : [params]
        const idx: number = arr[0]?.dataIndex ?? 0
        const raw = chartPoints.value[idx]?.date
        let dateStr = arr[0]?.axisValue as string
        if (raw) {
          const d = new Date(raw)
          const y = d.getFullYear()
          const mo = String(d.getMonth() + 1).padStart(2, '0')
          const day = String(d.getDate()).padStart(2, '0')
          const h = String(d.getHours()).padStart(2, '0')
          const min = String(d.getMinutes()).padStart(2, '0')
          dateStr = `${y}/${mo}/${day} ${h}:${min}`
        }
        const lines = arr.map((p: any) => `${p.marker}${p.seriesName ? `${p.seriesName}: ` : ''}${formatTooltip(p.data)}`)
        return `<div style="margin-bottom:4px;color:#b4c8dc;">${dateStr}</div>${lines.join('<br/>')}`
      },
    },

    xAxis: {
      type: 'category',
      data: labels,
      boundaryGap: false,
      axisLine: { show: false },
      axisTick: { show: false },
      splitLine: { show: false },
      axisLabel: {
        hideOverlap: true,
        color: axisText,
        fontSize: 10,
        margin: 10,
      },
    },

    yAxis: {
      type: 'value',
      axisLine: { show: false },
      axisTick: { show: false },
      axisLabel: {
        color: axisText,
        fontSize: 10,
        margin: 8,
        formatter: (v: number) => formatAxis(v),
      },
      splitLine: {
        show: true,
        lineStyle: { color: gridLine, type: 'dashed', width: 1 },
      },
    },

    series,
  }
})

function render() {
  if (!chart) { return }
  chart.setOption(option.value, { notMerge: false, lazyUpdate: true })
}

watch(option, async () => {
  await nextTick()
  chart?.resize()
  render()
})

// Init echarts lazily — only when el is actually visible (v-show = true)
const isChartVisible = computed(() => isLoading.value || chartPoints.value.length > 0 || pairChartPoints.value.length > 0)
watch(isChartVisible, async (visible) => {
  if (!visible || chart) { return }
  await nextTick()
  if (!el.value) { return }
  chart = echarts.init(el.value)
  render()
}, { immediate: true })

onBeforeUnmount(() => {
  if (import.meta.env.SSR) { return }
  chart?.dispose()
  chart = null
})

onMounted(() => {
  if (import.meta.env.SSR) { return }
  window.addEventListener('resize', () => chart?.resize())
})
</script>

<template>
  <div class="pool-statistic-card">
    <div class="pool-statistic-card__top">
      <div class="statistic-title">
        <div class="statistic-label">
          {{ cardLabel }}

          <j-loading-spinner
            v-if="isLoading && chartPoints.length === 0 && pairChartPoints.length === 0"
            width="14px"
            border-width="1.5px"
          />
        </div>
        <div class="statistic-value">
          {{ cardValue.formatted }}
        </div>

      </div>

      <div class="pool-statistic-card__top__opt">
        <j-select
          v-if="isShowSelect"
          v-model="currency"
          :options="currencyOptions"
          :unselected="false"
        />

        <chart-date-filter
          v-model="activeFilter"
          :filters="chartFilter.filters"
        />
      </div>
    </div>

    <div class="chart-el">

      <client-only>
        <div
          v-show="isLoading || chartPoints.length > 0 || pairChartPoints.length > 0"
          ref="el"
          style="height: 160px; width: 100%;"
        />
        <div
          v-if="!isLoading && chartPoints.length === 0 && pairChartPoints.length === 0"
          class="no-data"
        >
          No data
        </div>
      </client-only>
    </div>
  </div>
</template>

<style lang="scss">
.pool-statistic-card {
  background-color: $bg-card;
  border: 1px solid $border-primary;
  border-radius: 12px;
  min-width: 0;

  &__top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: $spacing-lg $spacing-xl;
    border-bottom: 1px solid $border-primary;
    position: relative;
    z-index: 1;

    @media (max-width: $breakpoint-xs) {
      align-items: flex-start;
    }

    &__opt {
      display: flex;
      gap: 6px;

      @media (max-width: $breakpoint-xs) {
        flex-direction: column;
      }
    }

    .statistic-label {
      font-size: 14px;
      color: $text-tertiary;
      position: relative;

      @media (max-width: $breakpoint-xs) {
        font-size: 12px;
      }

      .loading-spinner {
        position: absolute;
        top: 50%;
        left: 110%;
        transform: translateY(-50%);
      }
    }

    .statistic-value {
      font-size: 18px;
      font-family: $font-JetBrainsMono;

      @media (max-width: $breakpoint-xs) {
        font-size: 14px;
      }
    }

    .j-select {
      margin-left: auto;
      transform: translate(0, 1px);
      z-index: 1;

      .btn {
        height: 20px;
        font-size: 11px;
        padding: 4px 8px;
        border-radius: 4px;

        svg {
          width: 6px;
          height: 4px;
        }
      }

      .select-item {
        font-size: 12px;
      }
    }

    .j-btn-group {
      margin: 0;
    }
  }

  .chart-el {
    position: relative;
    padding: $spacing-lg $spacing-xl;
  }

  .no-data {
    width: 100%;
    height: 160px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    font-style: normal;
    font-weight: 400;
    line-height: 16px;
    color: $navi-50;
  }
}
</style>
