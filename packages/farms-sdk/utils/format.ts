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
  amount: string | number,
  decimals: number,
): bigint {
  const d = new Decimal(amount)

  const scaled = d.mul(
    new Decimal(10).pow(decimals),
  )

  return BigInt(
    scaled.toFixed(0, Decimal.ROUND_DOWN),
  )
}

export function bpsToNumber(bps: number): number {
  return bps / 10000
}
