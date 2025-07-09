export function normalizeAssetAmount(amount: number, decimals: number) {
    return amount / 10 ** decimals
}

export function amountToBigInt(amount: number, decimals: number) {
    return BigInt(amount * 10 ** decimals)
}