/**
 * Operator narration to **stderr** — colored, prefixed status lines.
 *
 * Strict separation from artifacts: anything a downstream tool would
 * pipe (XDRs, JSON snapshots) goes through `utils/io.ts` → stdout.
 * Everything in this file goes to stderr so `alula … > out.json`
 * captures only the artifact while the operator still sees progress.
 *
 * Color is auto-disabled when stderr is not a TTY (chalk handles it).
 */

import chalk from 'chalk'

export function info(msg: string): void {
  process.stderr.write(`${msg}\n`)
}

export function dim(msg: string): void {
  process.stderr.write(`${chalk.dim(msg)}\n`)
}

export function step(msg: string): void {
  process.stderr.write(`${chalk.cyan('›')} ${msg}\n`)
}

export function ok(msg: string): void {
  process.stderr.write(`${chalk.green('✓')} ${msg}\n`)
}

export function warn(msg: string): void {
  process.stderr.write(`${chalk.yellow('⚠')} ${msg}\n`)
}

export function fail(msg: string): never {
  process.stderr.write(`${chalk.red('✗')} ${msg}\n`)
  process.exit(1)
}

/**
 * Two-column key/value line for the `label: value` pattern repeated
 * across every command's preflight summary. Pads the label so multiple
 * `kv()` calls in a row line up.
 */
export function kv(label: string, value: string | number, labelWidth = 16): void {
  process.stderr.write(`${chalk.dim(`${label.padEnd(labelWidth)}`)}${value}\n`)
}

export function blank(): void {
  process.stderr.write('\n')
}
