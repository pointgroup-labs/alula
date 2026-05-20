import type { FunctionDef } from '../../types'

export const queueInMarketUpgrade: FunctionDef<{ new_wasm_hash: string }, { current_market_wasm_hash: string }> = {
  multisig: 'upgrade',
  contract: 'market_manager',
  function: 'queue_in_market_upgrade',
  id: 'market_manager.queue_in_market_upgrade',
  displayName: 'Queue Market WASM upgrade',
  description: 'Queues a new WASM hash for the Market contract. Once the on-chain timelock expires, the upgrade can be applied and will replace the code of every market deployed by this Market Manager.',
  argSchema: {
    new_wasm_hash: { kind: 'wasm-hash' },
  },
  // Not Stellar-timebounds-locked. The contract-side `apply_market_upgrade`
  // enforces the queue→apply delay via `queued_in_timestamp + UPGRADE_IN_QUEUE_SECONDS`;
  // setting `timebounds.minTime` on the *queue* tx would only delay when the
  // contract clock starts. Apply is permissionless and lives outside the catalog.
  isTimelocked: false,
  // Manager queues a single Market WASM hash that propagates to every
  // market it has spawned when apply runs. There is no per-market target.
  affectsAllMarkets: true,
  pairWith: {
    queue: 'market_manager.queue_in_market_upgrade',
    apply: 'market_manager.apply_in_market_upgrade',
    cancel: 'market_manager.cancel_market_upgrade',
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
