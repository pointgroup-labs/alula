import type { FunctionDef } from '../../types'

export const queueInMarketUpgrade: FunctionDef<{ new_wasm_hash: string }, { current_market_wasm_hash: string }> = {
  multisig: 'upgrade',
  contract: 'market_manager',
  function: 'queue_in_market_upgrade',
  id: 'market_manager.queue_in_market_upgrade',
  argSchema: {
    new_wasm_hash: { kind: 'wasm-hash' },
  },
  isTimelocked: true,
  pairWith: {
    queue: 'market_manager.queue_in_market_upgrade',
    apply: 'market_manager.apply_in_market_upgrade',
    cancel: 'market_manager.cancel_in_market_upgrade',
  },
  // fetchBeforeSnapshot is wired in once chain.ts lands; the page falls back
  // to args-only display if undefined.
  renderSummary: (args, snapshot) => ({
    title: 'Queue Market WASM upgrade',
    rows: [
      {
        label: 'Market WASM hash',
        before: snapshot?.current_market_wasm_hash,
        after: args.new_wasm_hash,
        severity: 'critical',
      },
    ],
  }),
}
