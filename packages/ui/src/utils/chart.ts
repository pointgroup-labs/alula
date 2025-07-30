import { SHORT_MONTHS } from '~/config'

export function normalizeChartDate(date: string) {
  const dateStr = new Date(date)
  const year = dateStr.getFullYear()
  const month = String(dateStr.getMonth() + 1).padStart(2, '0')
  const day = String(dateStr.getDate()).padStart(2, '0')
  return `${day}.${month}.${year}`
}

export function labelWithDateOrMonth(date: string, isMonth = false) {
  if (isMonth) {
    const month = new Date(date).getMonth()
    return SHORT_MONTHS[month]
  }
  return normalizeChartDate(date)
}
