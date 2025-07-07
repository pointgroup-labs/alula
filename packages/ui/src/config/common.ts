import { getRuntimeConfig } from '~/utils/config'

const config = getRuntimeConfig()

export const RPC_NETWORK = config.NUXT_PUBLIC_RPC || 'testnet'

// intervals
export const RELOAD_FEE_INTERVAL = 60_000

export const TEST_PUBKEY = 'GBLPWC4ULLYYXHK6BQY4M6G4DFRZRTOZGK5LMML32L3YP4DG3Z35P7OS'
