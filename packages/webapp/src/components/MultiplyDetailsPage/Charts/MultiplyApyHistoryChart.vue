<script lang="ts" setup>
import type { ECharts, EChartsOption } from 'echarts'
import * as echarts from 'echarts'
import { labelWithDateOrMonth, normalizeChartDate } from '~/utils/chart'

const MY_APY_COLOR = '#22d3ee'
const MIN_APY_COLOR = '#f59e0b'
const MAX_APY_COLOR = '#47cd89'

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

const mockMyApy = computed(() => generateMockData())
const mockMinApy = computed(() => generateMockData())
const mockMaxApy = computed(() => generateMockData())

const currentMyApy = computed(() => mockMyApy.value.at(-1)?.value ?? 0)
const currentMinApy = computed(() => mockMinApy.value.at(-1)?.value ?? 0)
const currentMaxApy = computed(() => mockMaxApy.value.at(-1)?.value ?? 0)

const { width } = useWindowSize()
const isMobile = computed(() => width.value <= 650)

const chartFilter = useChartFilter()
const activeFilter = toRef(chartFilter, 'activeFilter')

const maxY = ref(12)
const labels = ref<string[]>([])
const myApyValues = ref<number[]>([])
const minApyValues = ref<number[]>([])
const maxApyValues = ref<number[]>([])

watch([mockMyApy, mockMinApy, mockMaxApy, activeFilter], ([myApy, minApy, maxApy, filters]) => {
  if (!myApy || !minApy || !filters) { return }

  const fa = chartFilter.filterData(myApy) ?? []
  const fb = chartFilter.filterData(minApy) ?? []
  const fc = chartFilter.filterData(maxApy) ?? []

  labels.value = fa.map(i => i.timestamp)

  maxY.value = 12
  myApyValues.value = fa.map((i) => {
    maxY.value = Math.max(maxY.value, i.value)
    return i.value
  })
  minApyValues.value = fb.map((i) => {
    maxY.value = Math.max(maxY.value, i.value)
    return i.value
  })
  maxApyValues.value = fc.map((i) => {
    maxY.value = Math.max(maxY.value, i.value)
    return i.value
  })
}, { immediate: true })

// ---- ECharts init ----
const el = ref<HTMLElement | null>(null)
let chart: ECharts | null = null

const option = computed<EChartsOption>(() => {
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
        name: 'My APY',
        type: 'line',
        data: myApyValues.value,
        smooth: 0.45,
        showSymbol: false,
        lineStyle: { width: 2, color: MY_APY_COLOR, cap: 'round', join: 'round' },
        itemStyle: { color: MY_APY_COLOR },
        areaStyle: {
          opacity: 1,
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: 'rgba(34, 211, 238, 0.4)' },
            { offset: 1, color: 'rgba(34, 211, 238, 0)' },
          ]),
        },
      },
      {
        name: 'Min Multiplier',
        type: 'line',
        data: minApyValues.value,
        smooth: 0.45,
        showSymbol: false,
        lineStyle: { width: 2, color: MIN_APY_COLOR, cap: 'round', join: 'round' },
        itemStyle: { color: MIN_APY_COLOR },
        areaStyle: {
          opacity: 1,
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: 'rgba(245, 158, 11, 0.4)' },
            { offset: 1, color: 'rgba(245, 158, 11, 0)' },
          ]),
        },
      },
      {
        name: 'Max Multiplier',
        type: 'line',
        data: maxApyValues.value,
        smooth: 0.45,
        showSymbol: false,
        lineStyle: { width: 2, color: MAX_APY_COLOR, cap: 'round', join: 'round' },
        itemStyle: { color: MAX_APY_COLOR },
        areaStyle: {
          opacity: 1,
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: 'rgba(71, 205, 137, 0.4)' },
            { offset: 1, color: 'rgba(71, 205, 137, 0)' },
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
  <section id="multiply-apy-history-chart">
    <div class="stat-card">
      <div class="stat-card__header">
        <h3 class="title">
          Net APY History
        </h3>

        <div class="current-metrics-data">
          <metric-indicator
            :color="MY_APY_COLOR"
            label="My APY"
            :value="`${formatPrice(currentMyApy, 1, 2)}%`"
          />

          <metric-indicator
            :color="MIN_APY_COLOR"
            label="Min Multiplier"
            :value="`${formatPrice(currentMinApy, 1, 2)}%`"
          />

          <metric-indicator
            :color="MAX_APY_COLOR"
            label="Max Multiplier"
            :value="`${formatPrice(currentMaxApy, 1, 2)}%`"
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
section#multiply-apy-history-chart {
  h3 {
    margin: 0;
  }

  .current-metrics-data {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 12px;
    margin-left: 20px;

    @media (max-width: $breakpoint-xs) {
      margin: 6px auto;
      justify-content: center;
      row-gap: 4px;
    }
  }

  .chart-date-filters {
    margin-left: auto;

    @media (max-width: $breakpoint-xs) {
      margin: 0 auto;
    }
  }

  .stat-card__header {
    white-space: nowrap;
    flex-wrap: wrap;
    gap: 6px;
  }
}
</style>
