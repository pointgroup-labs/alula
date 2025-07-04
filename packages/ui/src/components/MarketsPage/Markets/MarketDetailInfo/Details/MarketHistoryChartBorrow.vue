<script lang="ts" setup>
import type { ChartData, ChartOptions } from 'chart.js'
import { normalizeChartDate } from '~/utils/chart'

// generate mock data
function generateMockData(): { date: string, value: number }[] {
  const data: { date: string, value: number }[] = []
  const today = new Date()
  const startDate = new Date()
  startDate.setMonth(today.getMonth() - 6)

  const currentDate = new Date(startDate)

  // eslint-disable-next-line no-unmodified-loop-condition
  while (currentDate <= today) {
    const dateStr = currentDate.toISOString().split('T')[0] // YYYY-MM-DD
    const value = +(Math.random() * (10 - 5) + 5).toFixed(1) // от 5.0 до 10.0 с 1 знаком после запятой
    data.push({ date: String(dateStr), value })

    currentDate.setDate(currentDate.getDate() + 1)
  }

  return data
}

const monthShirt = [
  'Jan', 'Feb', 'Mar', 'Apr', 'May', 'June', 'July', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec',
]

const mockData = computed(() => generateMockData())

const { width } = useWindowSize()

const isMobile = computed(() => width.value <= 650)

const filters = ['30 days', '1 month', '6 month']

const activeFilter = ref(filters[0])

const dataByFilters = computed(() => {
  const now = new Date()

  return [...mockData.value]
    ?.sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime())
    .filter((item) => {
      const itemDate = new Date(item.date)

      if (activeFilter.value === '30 days') {
        const refDate = new Date(now)
        refDate.setDate(now.getDate() - 30)
        return itemDate >= refDate && itemDate <= now
      }

      if (activeFilter.value === '1 month') {
        const refDate = new Date(now)
        refDate.setMonth(now.getMonth() - 1)
        return itemDate >= refDate && itemDate <= now
      }

      if (activeFilter.value === '6 month') {
        const refDate = new Date(now)
        refDate.setMonth(now.getMonth() - 6)
        return itemDate >= refDate && itemDate <= now
      }

      return false
    })
})

const maxY = ref(12)

const chartData = ref<ChartData<'line'>>({
  labels: [],
  datasets: [],
})

watch([
  dataByFilters,
  activeFilter,
], ([d, f]) => {
  if (!d || !f) {
    return
  }

  chartData.value.labels = d.map((item) => {
    if (f === '6 month') {
      const month = new Date(item.date).getMonth()
      return monthShirt[month]
    }
    return normalizeChartDate(item.date)
  })
  chartData.value.datasets = [
    {
      type: 'line',
      borderColor: '#FFD101',
      backgroundColor: '#FFD101',
      label: 'Borrow APR',
      data: d.map((item) => {
        maxY.value = Math.max(maxY.value, item.value)
        return item.value
      }),
    },
  ]
}, { immediate: true })

const chartOptions = computed<ChartOptions<'line'>>(() => {
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
    },
    plugins: {
      tooltip: {
        boxPadding: 4,
        usePointStyle: true,
        bodySpacing: 8,
        callbacks: {
          title(context: any) {
            const dataIndex = context[0]?.dataIndex
            const data = dataByFilters.value[dataIndex]
            if (!data?.date) {
              return context[0]?.label
            }
            return normalizeChartDate(data.date)
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
        Borrow History APR
      </div>
      <div class="history-chart__header__badge">
        AVG: 0.14%
      </div>

      <j-btn-group v-model="activeFilter" :buttons="filters" class="history-chart__header__filters" />
    </div>

    <div class="history-chart__chart">
      <custom-mixed-chart
        :key="activeFilter"
        :chart-data="chartData"
        :chart-options="chartOptions"
        :max-ticks-limit="6"
        chart-height="196px"
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

      &__filters {
        width: fit-content;
        margin-left: auto;
        border-radius: $spacing-4;

        .btn {
          width: fit-content;
          padding: $spacing-4 $spacing-8;
          border-radius: $spacing-4;

          .btn-content {
            font-size: 11px;
            font-style: normal;
            font-weight: 500;
            line-height: 12px;
          }
        }
        .btn-primary {
          background-color: $neutral-3;
          color: $dark;
          border-color: transparent;
        }
      }
    }

    &__chart {
      height: 196px;
    }
  }
}
</style>
