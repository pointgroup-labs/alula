// Main client
export * from './client'

// Services
export * from './services'

// Core
export * from './core'

// Config
export * from './config'

// Utilities
export * from './utils'

// Constants and types
export * from './constants'
export * from './types'

// Legacy exports for backward compatibility (deprecated)
// These will be removed in future versions
export { MarketService as MarketClient } from './services/market-service'
export { StellarClient } from './client'
