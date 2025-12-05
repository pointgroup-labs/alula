import Decimal from 'decimal.js'

/**
 * Function to allow only numbers and a single decimal point to be inputted.
 *
 * @param {any} e - event parameter
 */
export function onlyNumber(e: any) {
  const keyCode = e.keyCode ?? e.which
  if ((keyCode < 48 || keyCode > 57) && keyCode !== 46) {
    e.preventDefault()
  }
  if (keyCode === 46 && String(e.target.value).includes('.')) {
    e.preventDefault()
  }
}

export function formatPrice(price: number | string, minDigits = 0, maxDigits = 10) {
  const longPriceFormatter = new Intl.NumberFormat('en-US', {
    style: 'decimal',
    minimumFractionDigits: minDigits,
    maximumFractionDigits: maxDigits,
  })
  return longPriceFormatter.format(Number(price))
}

export function shortenNumber(num: number | string, digits = 2): string {
  const formatter = new Intl.NumberFormat('en', {
    notation: 'compact',
    compactDisplay: 'short',
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
  })
  return formatter.format(Number(num))
}

export function parseFormattedPrice(formattedPrice: string): number {
  return Number.parseFloat(formattedPrice.replaceAll(',', ''))
}

export function getZeroCountAfterDecimal(value: number | string): number {
  const str = Number(value).toExponential()
  const match = str.match(/e-(\d+)/)
  return match ? Number.parseInt(match[1]!, 10) : 0
}

// format number with decimals 99.9842 => 99.98
export function truncatePercent(value: number, dec = 2): string {
  const [intPart, decimalPart = ''] = value.toString().split('.')
  return dec === 2
    ? `${value.toFixed(2)}`
    : `${intPart}.${decimalPart.slice(0, dec)}`
}

export function bigintToNumber(
  rawValue: bigint,
  assetDecimals: number,
  outputDecimals: number = assetDecimals,
): string {
  if (!rawValue) {
    return '0'
  }
  const dec = new Decimal(rawValue.toString())
    .dividedBy(new Decimal(10).pow(assetDecimals))
  return dec.toFixed(outputDecimals)
}

export function destructurePoolAsset(poolAsset: string) {
  return poolAsset.split(':')
}

export function amountToUsdWithShort(amount: number, price: number, shorten = true) {
  const amountInUsd = Number(amount) * Number(price)
  if (shorten) {
    return shortenNumber(amountInUsd || 0)
  }
  return formatPrice(amountInUsd || 0, 2, 2)
}
