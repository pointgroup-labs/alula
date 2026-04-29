import type { FunctionDef } from '../../types'

export const cancelInManagerUpgrade: FunctionDef<Record<string, never>, { queued_wasm_hash: string | null }> = {
  multisig: 'upgrade',
  contract: 'market_manager',
  function: 'cancel_in_manager_upgrade',
  id: 'market_manager.cancel_in_manager_upgrade',
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
