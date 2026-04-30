#!/usr/bin/env -S npx tsx
/**
 * Alula operator CLI — `alula <group> <command> [flags]`.
 *
 * Routing + dispatch via commander. Each command group exports a
 * `*Commands(): Command` builder; this file wires them together.
 * `--network` is a global option declared on the root program; the
 * root `preAction` hook initializes the singleton context once before
 * any leaf action runs.
 */

import type { NetworkName } from './context'
import { Option, program } from 'commander'
import { description, version } from '../package.json'
import { multisigCommands } from './commands'
import { initContext } from './context'
import { fail } from './utils'

program
  .name('alula')
  .description(description)
  .version(version)
  .addOption(
    new Option('--network <name>', 'Stellar network')
      .choices(['testnet', 'public'])
      .env('NETWORK')
      .makeOptionMandatory(),
  )
  .hook('preAction', (root) => {
    initContext({ network: root.opts().network as NetworkName })
  })

program.addCommand(multisigCommands())

// Bare `alula` or `alula <group>` → translate to a help request so
// commander exits 0 instead of its default exit 1 (which pnpm decorates
// with an ELIFECYCLE banner on every help-discovery action).
const args = process.argv.slice(2)
const isBareGroup = args.length === 1 && program.commands.some(c => c.name() === args[0])
const finalArgs = args.length === 0 || isBareGroup ? [...args, '--help'] : args

program.parseAsync(finalArgs, { from: 'user' }).catch((err: unknown) => {
  fail(err instanceof Error ? err.message : String(err))
})
