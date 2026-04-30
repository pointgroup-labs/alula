import type { FunctionDef } from '../../types'

export const queueInMarketUpgrade: FunctionDef<{ new_wasm_hash: string }, { current_market_wasm_hash: string }> = {
  multisig: 'upgrade',
  contract: 'market_manager',
  function: 'queue_in_market_upgrade',
  id: 'market_manager.queue_in_market_upgrade',
  displayName: 'Queue Market WASM upgrade',
  description: 'Stages a new Market contract WASM hash. After the on-chain timelock elapses, anyone can apply the upgrade to deployed markets.',
  argSchema: {
    new_wasm_hash: { kind: 'wasm-hash' },
  },
  // Not Stellar-timebounds-locked. The contract-side `apply_market_upgrade`
  // enforces the queue→apply delay via `queued_in_timestamp + UPGRADE_IN_QUEUE_SECONDS`;
  // setting `timebounds.minTime` on the *queue* tx would only delay when the
  // contract clock starts. Apply is permissionless and lives outside the catalog.
  isTimelocked: false,
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
