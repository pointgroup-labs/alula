export const DEVNET_RPC_URL = 'https://horizon-futurenet.stellar.org'
export const TESTNET_RPC_URL = 'https://horizon-testnet.stellar.org'
export const PUBLIC_RPC_URL = 'https://horizon.stellar.org'

export const SOROBAN_TESTNET_RPC_URL = 'https://soroban-testnet.stellar.org'
export const SOROBAN_PUBLIC_RPC_URL = 'https://rpc.lightsail.network'

export const RPC_URLS: Record<string, string> = {
  testnet: TESTNET_RPC_URL,
  devnet: DEVNET_RPC_URL,
  public: PUBLIC_RPC_URL,
}

export const SOROBAN_RPC_URLS: Record<string, string> = {
  testnet: SOROBAN_TESTNET_RPC_URL,
  public: SOROBAN_PUBLIC_RPC_URL,
}

export const TESTNET_MARKET_CONTRACT_ID = 'CBAK5PM2PYW6MQ3M4A7542IAKD2SITBF6KXPDBAUP2WTBRZJPSZ5WKQZ'
export const PUBLIC_MARKET_CONTRACT_ID = 'CAAQXXQU4WUJEQT5OP6LXRWSBGQ6TMQUYLEII7U24OEMXYIQ3N2SBJVP'

export const SOROSWAP_PROVIDER_ADDRESS: Record<string, string> = {
  testnet: 'CCBMBQGNR3NCR2FPN3AYQMJ65UMGQHCOJUB34RQCGIYMX77D5HUUSQDG',
  public: 'CCN7CU6ZXTUWCW67T2BQWNMHUCEYBHQSAIPFYDXJWCNOTQQOWWJ3TOQN',
}

export const AQUA_PROVIDER_ADDRESS: Record<string, string> = {
  testnet: 'CAAMNEXA7BOLMJKHDWNWLW6NQONLW3D6EXIKBBDJJEIDOGJYXRD7PJG4',
  public: 'CAPSHFQDW4FAAICMWUZLVOBQVRF7OR36S72ZXKMOKNWPC5P57WULFKAL',
}

export const MAX_I128 = 170_141_183_460_469_231_731_687_303_715_884_105_727n

export const CONTRACT_ID: Record<string, string> = {
  testnet: TESTNET_MARKET_CONTRACT_ID,
  public: PUBLIC_MARKET_CONTRACT_ID,
}

export const SWAP_PROVIDERS: Record<string, Record<string, string>> = {
  aquarius: AQUA_PROVIDER_ADDRESS,
  soroswap: SOROSWAP_PROVIDER_ADDRESS,
}

export const VALID_PAIRS = new Set([
  'USDC-XLM',
  'AQUA-USDC',
])
