const SUCCESS_COLOR = '#17B26A'
const WARNING_COLOR = '#f0b100'
const DANGER_COLOR = '#f04438'

export function healthFactorColor(hf?: number | null): string {
  if (!hf) {
    return SUCCESS_COLOR
  }
  return hf < 1.2 ? DANGER_COLOR : (hf < 1.5 ? WARNING_COLOR : SUCCESS_COLOR)
}

export function ltvColor(current: number, max?: number) {
  if (!max) {
    return
  }
  const percentage = current / max
  return percentage < 0.7 ? SUCCESS_COLOR : (percentage < 0.9 ? WARNING_COLOR : DANGER_COLOR)
}

export function utilRateColor(rate?: number, limit?: number) {
  if (!rate) {
    return '#e8edf5'
  }
  const rateByLimit = limit ? rate / (limit || 100) * 100 : rate
  switch (true) {
    case rateByLimit >= 90: return DANGER_COLOR
    case rateByLimit >= 70: return WARNING_COLOR
    default: return 'rgb(0, 201, 80)'
  }
}
