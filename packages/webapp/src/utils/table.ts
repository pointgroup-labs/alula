import Decimal from 'decimal.js'

type Op = 'add' | 'sub' | 'mul' | 'div'

export function toDec(value: bigint): Decimal {
  return new Decimal(value.toString())
}

function operate(a: bigint, b: bigint, op: Op): Decimal {
  const da = toDec(a)
  const db = toDec(b)
  switch (op) {
    case 'add': return da.plus(db)
    case 'sub': return da.minus(db)
    case 'mul': return da.times(db)
    case 'div': return da.dividedBy(db)
  }
}

export function calcUserTotalShares(
  shares: bigint,
  totalShares: bigint,
  available: bigint,
  totalBorrowed: bigint,
  precision = 7,
): string {
  const fraction = operate(shares, totalShares, 'div')
  const totalLiq = operate(available, totalBorrowed, 'add')
  const raw = fraction.times(totalLiq)
  const scaled = raw.dividedBy(
    new Decimal(10).pow(precision),
  )
  return scaled.toFixed(precision)
}

export function checkIsCanUsePool(
  obligations: Iterable<[string, any]> | [],
  poolAddress?: string,
): boolean {
  if (!poolAddress) {
    return true
  }

  for (const [address] of obligations) {
    if (address === poolAddress) {
      return false
    }
  }

  return true
}
