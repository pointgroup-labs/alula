<script lang="ts" setup>
import type { ChartData, ChartOptions } from 'chart.js'
import {
  BarElement,
  CategoryScale,
  Chart,
  Filler,
  Legend,
  LinearScale,
  LineController,
  LineElement,
  PointElement,
  Tooltip,
} from 'chart.js'
import { Chart as MixedChart } from 'vue-chartjs'

const {
  chartData,
  chartOptions,
  chartWidth = '100%',
  chartHeight = '300px',
  maxTicksLimit = 16,
  useCustomBarWidthPlugin = false,
  useReverseBarPlugin = false,
  useVerticalLinesPlugin = false,
  useCustomLabels = false,
} = defineProps<{
  chartData: ChartData<'bar' | 'line'>
  chartOptions: ChartOptions<'bar' | 'line'>
  chartWidth?: string
  chartHeight?: string
  maxTicksLimit?: number
  useCustomBarWidthPlugin?: boolean
  useReverseBarPlugin?: boolean
  useVerticalLinesPlugin?: boolean
  useCustomLabels?: boolean
}>()

Chart.register(
  BarElement,
  CategoryScale,
  Filler,
  Legend,
  LinearScale,
  LineController,
  LineElement,
  PointElement,
  Tooltip,
)

const { width } = useWindowSize()

const hoverX = ref<number | null>(null)

const visibleTicks = ref([])

const options = computed<ChartOptions<'bar'>>(() => {
  const scalesX = {
    ...chartOptions?.scales?.x,
    ticks: {
      ...chartOptions?.scales?.x?.ticks,
      display: true,
      color: useCustomLabels ? 'transparent' : '#4E4E4E',
      maxTicksLimit: width.value > 1024 ? maxTicksLimit : 6,
      font: {
        size: useCustomLabels ? 1 : 11,
      },
    },
    grid: {
      display: false,
    },
    offset: !!chartOptions?.scales?.x?.offset,
  }

  return {
    ...chartOptions,
    scales: {
      ...chartOptions?.scales,
      x: scalesX,
    },
    plugins: {
      ...chartOptions?.plugins,
      legend: chartOptions?.plugins?.legend ?? {
        position: 'top',
        labels: {
          color: isDark.value ? '#fff' : '#878787',
          font: {
            size: width.value > 650 ? 12 : 10,
          },
          padding: 7,
          usePointStyle: true,
          pointStyle: 'circle',
          boxWidth: width.value > 650 ? 5 : 4,
          boxHeight: width.value > 650 ? 5 : 4,
        },
      },
    },

    onHover: (_event, elements) => {
      const x = elements[0]?.element?.x ?? 0
      hoverX.value = elements.length > 0 ? x : null
    },
  }
})

const hoverLinePlugin = {
  id: 'hoverLine',
  beforeDraw: (chart: Chart) => {
    if (hoverX.value !== null) {
      const ctx = chart.ctx
      ctx.save()
      ctx.beginPath()
      ctx.moveTo(hoverX.value, chart.chartArea.top)
      ctx.lineTo(hoverX.value, chart.chartArea.bottom)
      ctx.lineWidth = 1
      ctx.strokeStyle = isDark.value ? 'rgba(234, 234, 234, 0.1)' : 'rgba(234, 234, 234, 0.8)'
      ctx.stroke()
      ctx.restore()
    }
  },
}

const getFirstTickOffset = {
  id: 'getFirstTickOffset',
  afterLayout(chart: any) {
    const allTicksOffset = chart.scales.x.ticks.map((_t: any, idx: number) => chart.scales.x.getPixelForTick(idx))
    // const firstTickX = chart.scales.x.getPixelForTick(0) ?? 0

    // const secondPixel = chart.scales.x.getPixelForTick(1) ?? 0
    // const pixelSpacing = secondPixel - firstTickX

    visibleTicks.value = chart.scales.x.ticks.map((t: any, idx: number) => {
      // let offset = pixelSpacing
      // if (idx === 0) {
      //   offset = firstTickX
      // }
      return {
        ...t,
        pixelSpacing: allTicksOffset[idx],
      }
    })
  },
}

const customBarWidthPlugin = {
  id: 'customBarWidth',
  beforeDraw: (chart: Chart) => {
    const barDatasetIndex = chart.data.datasets.findIndex(dataset => dataset.type === 'bar')

    if (barDatasetIndex === -1) {
      return
    }

    const datasetMeta = chart.getDatasetMeta(barDatasetIndex)
    const bars = datasetMeta.data

    for (const [index, bar] of bars.entries()) {
      if (index === 0 || index === bars.length - 1) {
        // @ts-expect-error ...
        bar.width = (datasetMeta._dataset?.barThickness ?? 10) * 2
      }
    }
  },
}

const reverseBarPlugin = {
  id: 'reverseBarPlugin',
  beforeDraw: (chart: any) => {
    const barDatasetIndex = chart.data.datasets.findIndex((dataset: any) => dataset.type === 'bar')

    if (barDatasetIndex === -1) {
      return
    }

    const datasetMeta = chart.getDatasetMeta(barDatasetIndex)
    const bars = datasetMeta.data

    for (const bar of bars) {
      // TODO: axis (y1): from variable or detect
      bar.base = chart.scales.y1.getPixelForValue(chart.scales.y1.max)
    }
  },
}

const verticalLinesPlugin = {
  id: 'barLines',
  afterLayout() {},
  afterDatasetsDraw(chart: any) {
    const { ctx, chartArea, scales: { x } } = chart
    ctx.save()
    ctx.lineWidth = 1
    ctx.strokeStyle = isDark.value ? '#2F2F2F' : 'rgba(0, 0, 0, 0.05)'

    for (const [i, _] of chart.data.labels.entries()) {
      const xPos = x.getPixelForValue(i)
      ctx.beginPath()
      ctx.moveTo(xPos, chartArea.top)
      ctx.lineTo(xPos, chartArea.bottom)
      ctx.stroke()
    }

    ctx.restore()
  },
}

const plugins = [hoverLinePlugin, getFirstTickOffset]
if (useCustomBarWidthPlugin) {
  plugins.push(customBarWidthPlugin)
}
if (useReverseBarPlugin) {
  plugins.push(reverseBarPlugin)
}
if (useVerticalLinesPlugin) {
  plugins.push(verticalLinesPlugin)
}
</script>

<template>
  <mixed-chart
    type="bar"
    :data="chartData"
    :options="options"
    :plugins="plugins"
    :style="{ 'height': chartHeight, 'width': chartWidth, 'z-index': 1, 'position': 'relative' }"
    :min="0"
    @mouseleave="hoverX = null"
  />
  <custom-line-chart-legend
    v-if="useCustomLabels"
    :data="visibleTicks"
  />
</template>
