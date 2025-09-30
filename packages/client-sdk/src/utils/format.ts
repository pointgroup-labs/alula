import Decimal from 'decimal.js'

// export function normalizeAssetAmount(amount: number, decimals: number) {
//     return amount / 10 ** decimals
// }
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

export function amountToBigInt(
  amount: string,
  decimals: number,
): bigint {
  const [whole, frac = ''] = amount.split('.')
  const normalizedFrac = (frac + '0'.repeat(decimals)).slice(0, decimals)
  return BigInt(whole + normalizedFrac)
}
