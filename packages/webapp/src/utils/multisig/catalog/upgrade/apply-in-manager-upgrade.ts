import type { FunctionDef } from '../../types'

export const applyInManagerUpgrade: FunctionDef<Record<string, never>, { queued_wasm_hash: string }> = {
  multisig: 'upgrade',
  contract: 'market_manager',
  function: 'apply_in_manager_upgrade',
  id: 'market_manager.apply_in_manager_upgrade',
  argSchema: {},
  isTimelocked: true,
  renderSummary: (_args, snapshot) => ({
    title: 'Apply queued Market Manager WASM upgrade',
    rows: [
      {
        label: 'Queued WASM hash',
        after: snapshot?.queued_wasm_hash ?? '(read from chain at submit time)',
        severity: 'critical',
      },
    ],
  }),
}
