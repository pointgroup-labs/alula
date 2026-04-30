/**
 * Artifact I/O — **stdout** only. Used for things a downstream tool
 * would consume by piping (signed XDRs, JSON snapshots), and for
 * reading XDR envelopes piped in via stdin.
 */

export function emit(value: unknown): void {
  if (typeof value === 'string') {
    process.stdout.write(`${value}\n`)
  } else {
    process.stdout.write(`${JSON.stringify(value, null, 2)}\n`)
  }
}

export async function readStdin(): Promise<string> {
  if (process.stdin.isTTY) {
    return ''
  }
  const chunks: Buffer[] = []
  for await (const chunk of process.stdin) {
    chunks.push(typeof chunk === 'string' ? Buffer.from(chunk) : chunk)
  }
  return Buffer.concat(chunks).toString('utf8').trim()
}
