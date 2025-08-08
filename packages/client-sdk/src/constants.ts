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

export const SOROBAN_CONTRACT_ID = 'CDDF67Y2GJH7SWTXGQ4FSIC4NHNQNRJL4ITSQ5SUYJ77PGKORCFWYQGP'

export const CONTRACT_ID: Record<string, string> = {
  testnet: SOROBAN_CONTRACT_ID,
}
