<script lang="ts" setup>
import type {
  ChartOptions,
} from 'chart.js'
import { labelWithDateOrMonth } from '~/utils/chart'

type ChartDataset = {
  apy: number
  timestamp: Date
}

type MockedHistoryDataItem = {
  [key: string]: ChartDataset[]
}

const TOKEN_METRICS_OFFSET = [2, 3, 4, 5]

const MOCK_HISTORY_DATA: MockedHistoryDataItem[] = TOKEN_METRICS_OFFSET.map((el: any, index: number) => {
  const data = Array.from({ length: 180 }).map((_d, idx) => {
    const date = new Date()
    date.setDate(date.getDate() - idx)
    return {
      apy: 5 + index,
      timestamp: date,
    }
  })
  return {
    [el]: data,
  }
})

const colors = [
  '#00C4FF',
  '#03B46D',
  '#FFD600',
  '#FFB24D',
]

const LEGEND_COLORS = computed(() => {
  return TOKEN_METRICS_OFFSET.reduce((acc, el, index) => {
    acc[String(el)] = colors[index as number] ?? '#00C4FF'
    return acc
  }, {} as { [key: string]: string })
})

const { width } = useWindowSize()

const isMobile = computed(() => width.value <= 650)

const chartFilter = useChartFilter()
const activeFilter = toRef(chartFilter, 'activeFilter')

const maxY = ref(5)
const minY = ref(0)

const chartData = ref<any>({
  labels: [],
  datasets: [],
})

const historyData = computed(() => MOCK_HISTORY_DATA)

watch([
  historyData,
  activeFilter,
],
([history, _filter]) => {
  if (!history) {
    return
  }

  const labels: Array<string | Date> = []
  const data: { [key: string]: number[] } = {}

  for (let i = 0; i < history?.length; i++) {
    const item: any = history[i]

    const multiplier = Object.keys(item)[0]
    const values = chartFilter.filterData(Object.values(item).flat())

    const sortedPoints = values?.sort((a: ChartDataset, b: ChartDataset) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()) || []

    if (labels.length === 0) {
      for (let j = 0; j < sortedPoints?.length; j++) {
        const dayLabel = sortedPoints[j]?.timestamp
        labels.push(dayLabel)
      }
    }

    data[`multiplier_${multiplier}`] = sortedPoints?.map(p => p?.apy || 0)
  }

  chartData.value = {
    labels,
    datasets: TOKEN_METRICS_OFFSET?.map((el: any) => {
      return {
        type: 'line',
        label: String(el),
        borderColor: LEGEND_COLORS.value[String(el)],
        backgroundColor: LEGEND_COLORS.value[String(el)],
        data: data?.[`multiplier_${el}`] || [],
      }
    }),
  }

  maxY.value = Math.max(...Object.values(data).flat()) + 1 || 15
  minY.value = Math.min(...Object.values(data).flat()) - 1 || 0
}, { immediate: true },
)

const chartOptions = computed<ChartOptions<'line'>>(() => {
  return {
    responsive: true,
    maintainAspectRatio: false,
    borderWidth: 2,
    pointRadius: 0,
    pointHoverRadius: 4,
    interaction: {
      intersect: false,
      mode: 'index',
    },
    scales: {
      y: {
        position: 'left',
        max: Math.ceil(maxY.value),
        min: Math.floor(minY.value),
        ticks: {
          stepSize: 1,
          color: isDark.value ? '#5B5B5B' : '#111',
          callback: (value, _index, _ticks) => {
            if (_index === _ticks.length - 1) {
              return 'APY'
            }
            return `${value}%`
          },
          font: {
            size: isMobile.value ? 8 : 12,
          },
        },
        grid: {
          color: isDark.value ? '#2F2F2F' : '#EBEBEB',
          drawBorder: false,
        },
        border: {
          display: false,
        },
      },
      x: {
        ticks: {
          callback(value) {
            const rawLabel = this.getLabelForValue(Number(value))
            const withYear = activeFilter.value?.value !== 180 && width.value >= 650
            return labelWithDateOrMonth(rawLabel, activeFilter.value?.value === 180, withYear)
          },
        },
      },
    },
    plugins: {
      layout: {
        padding: {
          bottom: 0,
        },
      },
      legend: {
        display: false,
      },
      tooltip: {
        boxPadding: 4,
        usePointStyle: true,
        bodySpacing: 8,
        callbacks: {
          title(context) {
            return normalizeChartDate(context[0]?.label ?? '', true)
          },
          label(context) {
            return `${context.dataset.label}x: ${Number(context.raw).toFixed(2)}%`
          },
        },
      },
    },
  }
})
</script>

<template>
  <div class="multiply-chart">
    <div class="loop-multiply__title">
      Historical APR

      <chart-date-filter
        v-model="activeFilter"
        :filters="chartFilter.filters"
      />
    </div>

    <div class="loop-multiply__chart">
      <custom-mixed-chart
        :key="width"
        :chart-data="chartData"
        :chart-options="chartOptions"
        chart-height="196px"
        :max-ticks-limit="4"
      />
    </div>

    <div class="loop-multiply__legend">
      <div
        v-for="[key, color] in Object.entries(LEGEND_COLORS)?.sort((a, b) => Number(a[0]) - Number(b[0]))"
        :key="key"
        class="loop-legend-item"
      >
        <span :style="{ '--color': String(color) }" />
        {{ key }}x
      </div>
    </div>

    <div class="loop-multiply__vault hide-xs">
      <div class="loop-multiply__vault-title">
        Vault Info
      </div>

      <div class="loop-multiply__vault-info">
        Multiply uses one-click looping with a flash loan to boost your yield. Choose a multiplier to set leverage —
        higher multiplier means higher APY and higher liquidation risk. You can reduce or close the position at any
        time.
      </div>
    </div>
  </div>
</template>

<style lang="scss">
.multiply-chart {
  width: 500px;
  display: flex;
  flex-direction: column;
  gap: $spacing-16;

  @media (max-width: $breakpoint-sm) {
    width: 100%;
  }

  .loop-multiply__legend {
    display: flex;
    gap: $spacing-16;
  }

  .loop-legend-item {
    display: flex;
    align-items: center;
    gap: $spacing-4;
    color: $dark;
    font-size: 11px;
    font-style: normal;
    font-weight: 500;
    line-height: 12px;

    span {
      position: relative;
      width: 20px;
      height: 8px;

      &::before {
        content: '';
        width: 8px;
        height: 8px;
        border-radius: 50%;
        background-color: var(--color);
        position: absolute;
        left: 50%;
        transform: translateX(-50%);
      }

      &::after {
        content: '';
        width: 100%;
        height: 2px;
        background-color: var(--color);
        position: absolute;
        top: 50%;
        left: 0;
        transform: translatey(-50%);
      }
    }
  }
}

.loop-multiply__title {
  color: $dark;
  font-size: 12px;
  font-style: normal;
  font-weight: 700;
  line-height: 16px;
  display: flex;
  justify-content: space-between;
}

.loop-multiply__chart {
  height: 196px;
}

.loop-multiply__vault {
  display: flex;
  flex-direction: column;
  gap: $spacing-10;
  margin-top: auto;

  &-title {
    color: $dark;
    font-size: 12px;
    font-style: normal;
    font-weight: 700;
    line-height: 16px;
  }

  &-info {
    color: $neutral-16;
    font-size: 12px;
    font-style: normal;
    font-weight: 400;
    line-height: 16px;
  }
}

body.body--dark {
  .loop-multiply__vault-title {
    color: #fff;
  }

  .loop-multiply__vault-info {
    color: $neutral-7;
  }

  .loop-multiply__title {
    color: #fff;
  }
}
</style>
