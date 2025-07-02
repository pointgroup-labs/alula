import { Client } from 'sdk'

const TESTNET_CONTRACT_ID = 'CBVR4635CDX2YUP32PXL2TD6KEMWP7MDGL4T4OF43GHDCLZTRQNVKWJ6'
const TESTNET_RPC_URL = 'https://soroban-testnet.stellar.org'

enum Network {
  Mainnet = 'mainnet',
  Testnet = 'testnet',
}

export const NetworkPassphrase = {
  [Network.Mainnet]: 'Public Global Stellar Network ; September 2015',
  [Network.Testnet]: 'Test SDF Network ; September 2015',
}

async function main() {
  const client = new Client({
    rpcUrl: TESTNET_RPC_URL,
    contractId: TESTNET_CONTRACT_ID,
    networkPassphrase: NetworkPassphrase[Network.Testnet],
  })

  const res = await client.get_all_pools()

  const pool = await client.get_pool({
    pool_address: res.result[0],
  })
  console.log(pool.result)

  const decimals = await client.get_asset_decimals()
  console.log(decimals.result)

  const asset_price = await client.get_pool_asset_oracle_price({
    pool_address: res.result[0],
  })
  console.log(asset_price.result)
}

void main()
