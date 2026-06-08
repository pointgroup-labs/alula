import { SHORT_MONTHS } from '~/config'

export function normalizeChartDate(date: string, withYear = true): string {
  const dateStr = new Date(date)
  const year = dateStr.getFullYear()
  const month = String(dateStr.getMonth() + 1).padStart(2, '0')
  const day = String(dateStr.getDate()).padStart(2, '0')
  let res = `${day}.${month}`
  if (withYear) {
    res += `.${year}`
  }
  return res
}

export function chartDateHM(date: string) {
  const dateStr = new Date(date)
  const h = String(dateStr.getHours()).padStart(2, '0')
  const min = String(dateStr.getMinutes()).padStart(2, '0')
  return `${h}:${min}`
}

export function labelWithDateOrMonth(date: string, isMonth = false, withYear = true): string {
  if (isMonth) {
    const month = new Date(date).getMonth()
    return SHORT_MONTHS[month] || ''
  }
  return normalizeChartDate(date, withYear)
}
