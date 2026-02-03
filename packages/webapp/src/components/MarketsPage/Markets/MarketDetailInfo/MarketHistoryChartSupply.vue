<script lang="ts" setup>
import type { ChartData, ChartOptions } from 'chart.js'
import { labelWithDateOrMonth, normalizeChartDate } from '~/utils/chart'

// generate mock data
function generateMockData(): { timestamp: string, value: number }[] {
  const data: { timestamp: string, value: number }[] = []
  const today = new Date()
  const startDate = new Date()
  startDate.setMonth(today.getMonth() - 6)

  const currentDate = new Date(startDate)

  // eslint-disable-next-line no-unmodified-loop-condition
  while (currentDate <= today) {
    const dateStr = currentDate.toISOString().split('T')[0] // YYYY-MM-DD
    const value = +(Math.random() * (10 - 5) + 5).toFixed(1) // от 5.0 до 10.0 с 1 знаком после запятой
    data.push({ timestamp: String(dateStr), value })

    currentDate.setDate(currentDate.getDate() + 1)
  }

  return data
}

const mockData = computed(() => generateMockData())

const { width } = useWindowSize()

const isMobile = computed(() => width.value <= 650)

const chartFilter = useChartFilter()
const activeFilter = toRef(chartFilter, 'activeFilter')

const maxY = ref(12)

const chartData = ref<ChartData<'line'>>({
  labels: [],
  datasets: [],
})

watch([
  mockData,
  activeFilter,
], ([d, f]) => {
  if (!d || !f) {
    return
  }

  const filteredData = chartFilter.filterData(d)

  chartData.value.labels = filteredData?.map(item => item.timestamp)
  chartData.value.datasets = [
    {
      type: 'line',
      borderColor: isDark.value ? '#006CE4' : '#4dbef1',
      backgroundColor: isDark.value ? '#006CE4' : '#4dbef1',
      label: 'Supply APR',
      data: filteredData?.map((item) => {
        maxY.value = Math.max(maxY.value, item.value)
        return item.value
      }),
    },
  ]
}, { immediate: true })

const chartOptions = computed<ChartOptions<'bar' | 'line'>>(() => {
  return {
    responsive: true,
    maintainAspectRatio: false,
    fill: false,
    borderWidth: 2,
    pointRadius: 0,
    pointHoverRadius: 2,
    interaction: {
      intersect: false,
      mode: 'index',
    },
    elements: {
      line: {
        tension: 0.4,
        cubicInterpolationMode: 'monotone',
      },
    },
    scales: {
      y: {
        max: Math.ceil(maxY.value),
        ticks: {
          color: isDark.value ? '#5B5B5B' : '#4E4E4E',
          callback: (value, _index, _ticks) => `${value}%`,
          font: {
            size: isMobile.value ? 8 : 12,
          },
        },
        grid: {
          color: isDark.value ? '#2F2F2F' : '#DFE0E2',
          drawBorder: false,
        },
        border: {
          display: false,
          dash: [2, 2],
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
      tooltip: {
        boxPadding: 4,
        usePointStyle: true,
        bodySpacing: 8,
        callbacks: {
          title(context: any) {
            const date = context[0]?.label
            return normalizeChartDate(date, true)
          },
          label(context: any) {
            return context[0]?.label
          },
        },
      },
      legend: {
        display: false,
      },
    },
  }
})
</script>

<template>
  <div class="market-history-chart">
    <div class="history-chart__header">
      <div class="history-chart__header__title">
        Historical Supply APR
      </div>
      <div class="history-chart__header__badge">
        AVG: 0.14%
      </div>

      <chart-date-filter
        v-model="activeFilter"
        :filters="chartFilter.filters"
      />
    </div>

    <div class="history-chart__chart">
      <custom-mixed-chart
        :key="activeFilter.value"
        :chart-data="chartData"
        :chart-options="chartOptions"
        :max-ticks-limit="6"
        chart-height="260px"
      />
    </div>
  </div>
</template>

<style lang="scss">
.market-history-chart {
  display: flex;
  flex-direction: column;
  gap: $spacing-16;

  .history-chart {
    &__header {
      display: flex;
      align-items: center;
      gap: $spacing-8;

      &__title {
        font-size: 12px;
        font-style: normal;
        font-weight: 700;
        line-height: 16px;
      }

      &__badge {
        font-size: 12px;
        font-style: normal;
        font-weight: 500;
        line-height: 16px;
        padding: $spacing-4 $spacing-12;
        border-radius: $spacing-4;
        background-color: $neutral-2;
      }
    }

    &__chart {
      height: 260px;
    }
  }
}

.theme-dark {
  .market-history-chart {
    .history-chart__header__badge {
      background: $neutral-18;
      color: $neutral-9;
    }
  }
}
</style>
