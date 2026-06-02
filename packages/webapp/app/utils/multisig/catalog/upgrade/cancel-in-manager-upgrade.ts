import type { FunctionDef } from '../../types'

export const cancelInManagerUpgrade: FunctionDef<Record<string, never>, { queued_wasm_hash: string | null }> = {
  multisig: 'upgrade',
  contract: 'market_manager',
  function: 'cancel_manager_upgrade',
  id: 'market_manager.cancel_manager_upgrade',
  displayName: 'Cancel queued Market Manager WASM upgrade',
  description: 'Removes the queued Market Manager WASM upgrade before it can be applied. Use this to abort a queued upgrade while the timelock is still running.',
  argSchema: {},
  isTimelocked: false,
  renderSummary: (_args, snapshot) => ({
    title: 'Cancel queued Market Manager WASM upgrade',
    rows: [
      {
        label: 'Currently queued',
        after: snapshot?.queued_wasm_hash ?? '(none)',
        severity: 'warning',
      },
    ],
  }),
}
