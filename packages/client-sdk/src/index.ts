// Main client
export * from './client'

export { StellarClient } from './client'

// Config
export * from './config'

// Constants and types
export * from './constants'

// Core
export * from './core'

// Services
export * from './services'
// Legacy exports for backward compatibility (deprecated)
// These will be removed in future versions
export { MarketService as MarketClient } from './services/market'

export * from './types'
// Utilities
export * from './utils'
