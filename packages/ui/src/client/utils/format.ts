export function normalizeAssetAmount(amount: number, decimals: number) {
    return amount / 10 ** decimals
}
