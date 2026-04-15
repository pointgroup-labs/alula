export const DEVNET_RPC_URL = 'https://horizon-futurenet.stellar.org'
export const TESTNET_RPC_URL = 'https://horizon-testnet.stellar.org'
export const PUBLIC_RPC_URL = 'https://horizon.stellar.org'

export const SOROBAN_TESTNET_RPC_URL = 'https://soroban-testnet.stellar.org'
export const SOROBAN_PUBLIC_RPC_URL = 'https://soroban.stellar.org'

export const RPC_URLS: Record<string, string> = {
  testnet: TESTNET_RPC_URL,
  devnet: DEVNET_RPC_URL,
  public: PUBLIC_RPC_URL,
}

export const SOROBAN_RPC_URLS: Record<string, string> = {
  testnet: SOROBAN_TESTNET_RPC_URL,
  public: SOROBAN_PUBLIC_RPC_URL,
}

export const MARKET_CONTRACT_ID = 'CDVBJCHFI62K3R5I6LOVKDDEK7WS4D6LVSJSG747K6GKL622E2ETFAMO'

export const SOROSWAP_PROVIDER_ADDRESS = 'CCYUQMA3KYXEQWGZM5PIJQVZR3ZMBXP56LBTT5LDN3JBP2WG6RUBSFYN'

export const MAX_I128 = 170_141_183_460_469_231_731_687_303_715_884_105_727n

export const CONTRACT_ID: Record<string, string> = {
  testnet: MARKET_CONTRACT_ID,
}
