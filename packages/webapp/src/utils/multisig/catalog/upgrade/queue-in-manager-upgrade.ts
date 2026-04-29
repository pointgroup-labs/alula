import type { FunctionDef } from '../../types'

export const queueInManagerUpgrade: FunctionDef<{ new_wasm_hash: string }, { current_manager_wasm_hash: string }> = {
  multisig: 'upgrade',
  contract: 'market_manager',
  function: 'queue_in_manager_upgrade',
  id: 'market_manager.queue_in_manager_upgrade',
  argSchema: {
    new_wasm_hash: { kind: 'wasm-hash' },
  },
  isTimelocked: true,
  pairWith: {
    queue: 'market_manager.queue_in_manager_upgrade',
    apply: 'market_manager.apply_in_manager_upgrade',
    cancel: 'market_manager.cancel_in_manager_upgrade',
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
