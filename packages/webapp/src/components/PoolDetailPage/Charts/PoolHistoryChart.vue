<script lang="ts" setup>
import type { ECharts, EChartsOption } from 'echarts'
import * as echarts from 'echarts'
import { labelWithDateOrMonth, normalizeChartDate } from '~/utils/chart'

function generateMockData(): { timestamp: string, value: number }[] {
  const data: { timestamp: string, value: number }[] = []
  const today = new Date()
  const startDate = new Date()
  startDate.setMonth(today.getMonth() - 6)

  const currentDate = new Date(startDate)
  // eslint-disable-next-line no-unmodified-loop-condition
  while (currentDate <= today) {
    const dateStr = currentDate.toISOString().split('T')[0]
    const value = +(Math.random() * (10 - 5) + 5).toFixed(1)
    data.push({ timestamp: String(dateStr), value })
    currentDate.setDate(currentDate.getDate() + 1)
  }
  return data
}

const mockSupply = computed(() => generateMockData())
const mockBorrow = computed(() => generateMockData())

const currentSupplyApy = computed(() => mockSupply.value.at(-1)?.value ?? 0)
const currentBorrowApy = computed(() => mockBorrow.value.at(-1)?.value ?? 0)

const { width } = useWindowSize()
const isMobile = computed(() => width.value <= 650)

const chartFilter = useChartFilter()
const activeFilter = toRef(chartFilter, 'activeFilter')

const maxY = ref(12)
const labels = ref<string[]>([])
const supplyValues = ref<number[]>([])
const borrowValues = ref<number[]>([])

watch([mockSupply, mockBorrow, activeFilter], ([s, b, f]) => {
  if (!s || !b || !f) { return }

  const fs = chartFilter.filterData(s) ?? []
  const fb = chartFilter.filterData(b) ?? []

  labels.value = fs.map(i => i.timestamp)

  maxY.value = 12
  supplyValues.value = fs.map((i) => {
    maxY.value = Math.max(maxY.value, i.value)
    return i.value
  })
  borrowValues.value = fb.map((i) => {
    maxY.value = Math.max(maxY.value, i.value)
    return i.value
  })
}, { immediate: true })

// ---- ECharts init ----
const el = ref<HTMLElement | null>(null)
let chart: ECharts | null = null

const option = computed<EChartsOption>(() => {
  const supply = '#22d3ee'
  const borrow = '#8a8df4'

  const gridLine = 'rgba(120, 160, 200, 0.18)'
  const axisText = 'rgba(180, 200, 220, 0.55)'

  return {
    backgroundColor: 'transparent',
    animation: true,
    animationDurationUpdate: 400,
    animationEasingUpdate: 'cubicOut',

    grid: { left: 0, right: 10, top: 0, bottom: 12 },

    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'line',
        lineStyle: { color: gridLine, type: 'dashed' },
      },
      padding: 10,
      backgroundColor: 'rgba(10,14,23,0.92)',
      borderColor: 'rgba(255,255,255,0.06)',
      borderWidth: 1,
      textStyle: { color: '#E8EDF5', fontSize: 12 },
      formatter: (params: any) => {
        const arr = Array.isArray(params) ? params : [params]
        const dateRaw = arr[0]?.axisValue as string
        const title = normalizeChartDate(dateRaw, true)
        const lines = arr.map((p: any) => `${p.marker}${p.seriesName}: <b">${p.data}%</b>`)
        return `<div style="margin-bottom:6px;">${title}</div>${lines.join('<br/>')}`
      },
    },

    xAxis: {
      type: 'category',
      data: labels.value,
      boundaryGap: false,
      axisLine: { show: false },
      axisTick: { show: false },
      splitLine: { show: false },
      axisLabel: {
        hideOverlap: true,
        color: axisText,
        fontSize: isMobile.value ? 10 : 12,
        margin: 14,
        formatter: (v: string) => {
          return labelWithDateOrMonth(v, activeFilter.value?.value === 180, false)
        },
      },
    },

    yAxis: {
      type: 'value',
      min: 0,
      max: Math.ceil(maxY.value),
      axisLine: { show: false },
      axisTick: { show: false },
      axisLabel: {
        color: axisText,
        fontSize: isMobile.value ? 10 : 12,
        margin: 18,
        formatter: (v: number) => `${v}%`,
      },
      splitLine: {
        show: true,
        lineStyle: {
          color: gridLine,
          type: 'dashed',
          width: 1,
        },
      },
    },

    series: [
      {
        name: 'Supply APR',
        type: 'line',
        data: supplyValues.value,
        smooth: 0.45,
        showSymbol: false,
        lineStyle: { width: 2, color: supply, cap: 'round', join: 'round' },
        itemStyle: { color: supply },
        areaStyle: {
          opacity: 1,
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: 'rgba(34, 211, 238, 0.4)' },
            { offset: 1, color: 'rgba(34, 211, 238, 0)' },
          ]),
        },
      },
      {
        name: 'Borrow APR',
        type: 'line',
        data: borrowValues.value,
        smooth: 0.45,
        showSymbol: false,
        lineStyle: { width: 2, color: borrow, cap: 'round', join: 'round' },
        itemStyle: { color: borrow },
        areaStyle: {
          opacity: 1,
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: 'rgba(138, 142, 244, 0.4)' },
            { offset: 1, color: 'rgba(138, 142, 244, 0)' },
          ]),
        },
      },
    ],
  }
})

function render() {
  if (!chart) {
    return
  }
  chart.setOption(option.value, { notMerge: false, lazyUpdate: true })
}

watch(option, () => {
  render()
})

onBeforeUnmount(() => {
  if (import.meta.env.SSR) {
    return
  }
  chart?.dispose()
  chart = null
})

onMounted(async () => {
  if (import.meta.env.SSR) {
    return
  }
  await nextTick()
  if (!el.value) {
    return
  }

  chart = echarts.init(el.value)
  render()

  window.addEventListener('resize', () => chart?.resize())
})
</script>

<template>
  <section id="market-history-chart">
    <div class="stat-card">
      <div class="stat-card__header">
        <h3 class="title">
          Interest Rate History
        </h3>

        <div class="current-metrics-data">
          <metric-indicator
            color="#22d3ee"
            label="Supply"
            :value="`${formatPrice(currentSupplyApy, 1, 2)}%`"
          />

          <metric-indicator
            color="#8a8df4"
            label="Borrow"
            :value="`${formatPrice(currentBorrowApy, 1, 2)}%`"
          />
        </div>

        <chart-date-filter
          v-model="activeFilter"
          :filters="chartFilter.filters"
        />
      </div>

      <div class="stat-card__body">
        <client-only>
          <div class="history-chart">
            <div
              ref="el"
              style="height: 200px; width: 100%;"
            />
          </div>
        </client-only>
      </div>
    </div>
  </section>
</template>

<style lang="scss">
section#market-history-chart {
  .current-metrics-data {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-left: 20px;
  }

  .chart-date-filters {
    margin-left: auto;
  }
}
</style>
