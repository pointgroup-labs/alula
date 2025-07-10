import Decimal from 'decimal.js'
import { defineStore } from 'pinia'

function toDec(value: bigint): Decimal {
    return new Decimal(value.toString())
}

type Op = 'add' | 'sub' | 'mul' | 'div'
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

function calcUserTotalShares(
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

export const useUserStore = defineStore('user', () => {
    const wallet = useWallet()
    const marketsStore = useMarketsStore()

    const clientStore = useClientStore()
    const jLendClient = computed(() => clientStore.jLendClient)

    const userObligation = ref()

    async function loadUserObligation() {
        userObligation.value = await jLendClient.value.sdk.getUserObligation(wallet.publicKey)
        console.log('%c[User Obligation]', 'color: #FFB726', userObligation.value)
    }

    const userBorrowAvailableInUsd = computed(() => {
        const deposits = userObligation.value?.deposits
        if (!deposits) {
            return 0
        }

        const assetDecimals = jLendClient.value.sdk.assetDecimals

        let userAvailableInUsd = 0

        // eslint-disable-next-line unicorn/no-for-loop
        for (let i = 0; i < deposits.length; i++) {
            const deposit = deposits[i]
            const [depositedPoolAddress, data] = deposit
            const depositedPool = marketsStore.state.pollsData?.find(p => p.pool_address === depositedPoolAddress)

            const userShares = data?.shares
            if (!depositedPool || !userShares) {
                userAvailableInUsd += 0
                continue
            }
            const userAvailable = calcUserTotalShares(userShares, depositedPool.total_shares, depositedPool?.available, depositedPool?.total_borrowed, assetDecimals)
            const availableInUsd = Number(userAvailable) * Number(depositedPool.pool_price)
            userAvailableInUsd += availableInUsd || 0
        }
        return userAvailableInUsd
    })

    watch(() => wallet.publicKey, async (p) => {
        if (!p) {
            userObligation.value = undefined
            return
        }
        await loadUserObligation()
    })

    return {
        userObligation,
        userBorrowAvailableInUsd,

        loadUserObligation,

    }
})
