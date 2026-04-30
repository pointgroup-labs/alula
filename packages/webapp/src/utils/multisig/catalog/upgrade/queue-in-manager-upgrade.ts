import type { FunctionDef } from '../../types'

export const queueInManagerUpgrade: FunctionDef<{ new_wasm_hash: string }, { current_manager_wasm_hash: string }> = {
  multisig: 'upgrade',
  contract: 'market_manager',
  function: 'queue_in_manager_upgrade',
  id: 'market_manager.queue_in_manager_upgrade',
  displayName: 'Queue Market Manager WASM upgrade',
  description: 'Stages a new Market Manager contract WASM hash. After the on-chain timelock elapses, anyone can apply the manager upgrade.',
  argSchema: {
    new_wasm_hash: { kind: 'wasm-hash' },
  },
  // Not Stellar-timebounds-locked. The contract-side `apply_manager_upgrade`
  // enforces the queue→apply delay; setting `timebounds.minTime` on the queue
  // tx would only delay when the contract clock starts. Apply is permissionless
  // and lives outside the catalog.
  isTimelocked: false,
  pairWith: {
    queue: 'market_manager.queue_in_manager_upgrade',
    apply: 'market_manager.apply_in_manager_upgrade',
    cancel: 'market_manager.cancel_manager_upgrade',
  },
  renderSummary: (args, snapshot) => ({
    title: 'Queue Market Manager WASM upgrade',
    rows: [
      {
        label: 'Market Manager WASM hash',
        before: snapshot?.current_manager_wasm_hash,
        after: args.new_wasm_hash,
        severity: 'critical',
      },
    ],
  }),
}
