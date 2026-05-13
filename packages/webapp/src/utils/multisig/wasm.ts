/**
 * WASM-hash on-chain verification.
 *
 * Resolves a claimed hash against the live ledger, fetches the wasm
 * bytes, and surfaces enough structural signal (function exports,
 * custom sections, SDK interface version when present) that an operator
 * can tell whether the hash refers to the kind of contract they intend
 * to upgrade.
 */

import { rpc, xdr } from '@stellar/stellar-sdk'

export type WasmCustomSection = {
  name: string
  byte_size: number
}

export type OnChainWasmInfo = {
  /** True iff a ContractCode ledger entry exists for the hash. */
  exists: boolean
  /** Total wasm byte length. Undefined if !exists. */
  byte_size?: number
  /**
   * Function-export names declared by the wasm module. Soroban
   * contracts export their public entry points by name — operators
   * use this list to confirm the hash refers to the *kind* of contract
   * they meant to queue.
   */
  exports?: string[]
  /**
   * Custom sections present in the wasm. Soroban contracts carry
   * `contractenvmetav0` (SDK version), `contractspecv0_<fn>` (per-
   * function XDR spec), and optionally `contractmetav0` (author-
   * supplied k/v metadata). Their presence and naming pattern is an
   * additional fingerprint.
   */
  custom_sections?: WasmCustomSection[]
  /**
   * Best-effort decode of the `contractenvmetav0` section. Format is
   * stable enough across SDK versions that we can pull the interface
   * version pair; we surface it as a single string. Null if the section
   * is missing or the decode fails (non-fatal).
   */
  sdk_interface_version?: string | null
  /** Surface message from the RPC or wasm parser. Non-fatal — UI decides. */
  parse_error?: string
}

const HEX_64_RE = /^[0-9a-f]{64}$/

export async function verifyHashOnChain(
  rpcUrl: string,
  hashHex: string,
): Promise<OnChainWasmInfo> {
  const hex = hashHex.trim().toLowerCase()
  if (!HEX_64_RE.test(hex)) {
    return { exists: false, parse_error: 'hash must be 64 lowercase hex characters' }
  }

  const server = new rpc.Server(rpcUrl)
  const key = xdr.LedgerKey.contractCode(
    new xdr.LedgerKeyContractCode({ hash: Buffer.from(hex, 'hex') }),
  )

  let entries: Awaited<ReturnType<typeof server.getLedgerEntries>>
  try {
    entries = await server.getLedgerEntries(key)
  } catch (error) {
    return { exists: false, parse_error: (error as Error).message ?? String(error) }
  }
  if (entries.entries.length === 0) {
    return { exists: false }
  }

  const val = entries.entries[0]!.val
  if (val.switch().name !== 'contractCode') {
    return { exists: false, parse_error: `unexpected ledger entry type: ${val.switch().name}` }
  }
  const codeBytes = new Uint8Array(val.contractCode().code())

  // Run the wasm-bytes inspection inside its own try/catch so a parse
  // failure still returns the exists+size data — operators can fall
  // back to comparing the hash itself even when introspection fails.
  let exports: string[] | undefined
  let customSections: WasmCustomSection[] | undefined
  let sdkVersion: string | null | undefined
  let parseError: string | undefined
  try {
    const mod = await WebAssembly.compile(codeBytes)
    exports = WebAssembly.Module.exports(mod)
      .filter(e => e.kind === 'function')
      .map(e => e.name)

    customSections = listCustomSections(mod, codeBytes)
    sdkVersion = decodeSdkInterfaceVersion(mod)
  } catch (error) {
    parseError = `wasm parse failed: ${(error as Error).message ?? String(error)}`
  }

  return {
    exists: true,
    byte_size: codeBytes.byteLength,
    exports,
    custom_sections: customSections,
    sdk_interface_version: sdkVersion,
    parse_error: parseError,
  }
}

/**
 * `WebAssembly.Module.customSections(mod, name)` only returns sections
 * matching a known name. Soroban embeds *unknown-to-us* sections
 * (e.g. one per spec entry), so we re-walk the binary to enumerate
 * every custom section. The format is the standard wasm preamble +
 * section-id-0 (custom) frames, length-prefixed.
 */
function listCustomSections(mod: WebAssembly.Module, bytes: Uint8Array): WasmCustomSection[] {
  // Cheap path first: collect bytes-sizes for any name the consumer
  // might already know about. We still walk the binary below to find
  // the others, but if WebAssembly.Module.customSections gives us a
  // hit on a well-known name we trust it and skip re-decoding.
  const known = ['contractenvmetav0', 'contractmetav0']
  const knownSizes = new Map<string, number>()
  for (const name of known) {
    const matches = WebAssembly.Module.customSections(mod, name)
    if (matches.length > 0 && matches[0]) {
      knownSizes.set(name, matches[0].byteLength)
    }
  }

  const result: WasmCustomSection[] = []
  // Skip the 8-byte wasm preamble (magic + version).
  let offset = 8
  const decoder = new TextDecoder('utf-8', { fatal: false })
  while (offset < bytes.byteLength) {
    const sectionId = bytes[offset]!
    offset += 1
    const { value: sectionLen, next: afterLen } = readVarUint(bytes, offset)
    offset = afterLen
    const sectionEnd = offset + sectionLen
    if (sectionId === 0) {
      // Custom section: name is length-prefixed UTF-8 at the start.
      const { value: nameLen, next: afterNameLen } = readVarUint(bytes, offset)
      const nameStart = afterNameLen
      const nameEnd = nameStart + nameLen
      const name = decoder.decode(bytes.subarray(nameStart, nameEnd))
      // Section bytes include the name; subtract it for a cleaner size.
      const payloadSize = sectionEnd - nameEnd
      result.push({ name, byte_size: payloadSize })
    }
    offset = sectionEnd
  }

  // Honor the trusted sizes from WebAssembly's own decoder where we have them.
  for (const s of result) {
    const trusted = knownSizes.get(s.name)
    if (trusted !== undefined) { s.byte_size = trusted }
  }
  return result
}

/**
 * Reads a wasm LEB128-encoded varuint32. Returns the decoded value and
 * the offset just past the encoded bytes. Bounded by 5 bytes per spec.
 */
function readVarUint(bytes: Uint8Array, offset: number): { value: number, next: number } {
  let result = 0
  let shift = 0
  let pos = offset
  for (let i = 0; i < 5; i++) {
    const b = bytes[pos]!
    pos += 1
    result |= (b & 0x7F) << shift
    if ((b & 0x80) === 0) { return { value: result >>> 0, next: pos } }
    shift += 7
  }
  return { value: result >>> 0, next: pos }
}

/**
 * Soroban embeds the SDK interface version as XDR
 * (`ScEnvMetaEntry::ScEnvMetaKindInterfaceVersion`) inside the
 * `contractenvmetav0` custom section. We try the SDK decode and fall
 * back gracefully if the layout has shifted.
 */
function decodeSdkInterfaceVersion(mod: WebAssembly.Module): string | null {
  const sections = WebAssembly.Module.customSections(mod, 'contractenvmetav0')
  if (sections.length === 0 || !sections[0]) { return null }
  try {
    const entry = xdr.ScEnvMetaEntry.fromXDR(Buffer.from(new Uint8Array(sections[0])), 'raw')
    const kind = entry.switch().name
    if (kind === 'scEnvMetaKindInterfaceVersion') {
      const v = entry.interfaceVersion()
      // `v` is an object with `protocol` and `preRelease` u32-or-u64 fields,
      // depending on SDK version. Render permissively.
      const protocol = typeof v.protocol === 'function' ? v.protocol() : (v as { protocol?: unknown }).protocol
      const preRelease = typeof v.preRelease === 'function' ? v.preRelease() : (v as { preRelease?: unknown }).preRelease
      if (protocol != null) { return preRelease != null ? `${protocol}.${preRelease}` : String(protocol) }
    }
  } catch {
    return null
  }
  return null
}
