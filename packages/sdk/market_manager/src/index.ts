import { Buffer } from "buffer";
import { Address } from '@stellar/stellar-sdk';
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from '@stellar/stellar-sdk/contract';
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Typepoint,
  Duration,
} from '@stellar/stellar-sdk/contract';
export * from '@stellar/stellar-sdk'
export * as contract from '@stellar/stellar-sdk/contract'
export * as rpc from '@stellar/stellar-sdk/rpc'

if (typeof window !== 'undefined') {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}




/**
 * Market Manager Contract Error
 */
export const MMCError = {
  0: {message:"InternalError"},
  1: {message:"MarketAlreadyExists"}
}

export type DataKey = {tag: "Config", values: void} | {tag: "MarketList", values: void};


export interface Config {
  admin: string;
  market_contract_wasm_hash: Buffer;
}

export interface Client {
  /**
   * Construct and simulate a deploy transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deploy: ({salt, market_admin, name, oracle}: {salt: Buffer, market_admin: string, name: string, oracle: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<string>>>

  /**
   * Construct and simulate a get_market_list transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_market_list: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Array<string>>>

  /**
   * Construct and simulate a upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Upgrades the market manager contract
   * 
   * ### Arguments
   * * `new_wasm_hash` - hash of the WASM binary uploaded to the network that will be used as a
   * new version of the contract
   */
  upgrade: ({new_wasm_hash}: {new_wasm_hash: Buffer}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a upgrade_deployed_markets transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Upgrades all deployed market contracts
   * 
   * ### Arguments
   * * `new_market_contract_wasm_hash` - hash of the WASM binary uploaded to the network that
   * will be used as a new version of the contract for every deployed market
   */
  upgrade_deployed_markets: ({new_market_contract_wasm_hash}: {new_market_contract_wasm_hash: Buffer}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {admin, market_contract_wasm_hash}: {admin: string, market_contract_wasm_hash: Buffer},
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy({admin, market_contract_wasm_hash}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAAAAAAAGZGVwbG95AAAAAAAEAAAAAAAAAARzYWx0AAAD7gAAACAAAAAAAAAADG1hcmtldF9hZG1pbgAAABMAAAAAAAAABG5hbWUAAAAQAAAAAAAAAAZvcmFjbGUAAAAAABMAAAABAAAD6QAAABMAAAfQAAAACE1NQ0Vycm9y",
        "AAAAAAAAAAAAAAAPZ2V0X21hcmtldF9saXN0AAAAAAAAAAABAAAD6gAAABM=",
        "AAAAAAAAANdDb25zdHJ1Y3RzIHRoZSBtYW5hZ2VyIGNvbnRyYWN0CgojIyMgQXJndW1lbnRzCiogYGFkbWluYCAtIG1hbmFnZXIncyBhZG1pbgoqIGBtYXJrZXRfY29udHJhY3Rfd2FzbV9oYXNoYCAtIGhhc2ggb2YgdGhlIFdBU00gYmluYXJ5IHVwbG9hZGVkIHRvIHRoZSBuZXR3b3JrLCB1c2VkIGFzIGEKdmVyc2lvbiBvZiB0aGUgZGVwbG95ZWQgbWFya2V0IGNvbnRyYWN0IGluc3RhbmNlcwAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAZbWFya2V0X2NvbnRyYWN0X3dhc21faGFzaAAAAAAAA+4AAAAgAAAAAA==",
        "AAAAAAAAAKpVcGdyYWRlcyB0aGUgbWFya2V0IG1hbmFnZXIgY29udHJhY3QKCiMjIyBBcmd1bWVudHMKKiBgbmV3X3dhc21faGFzaGAgLSBoYXNoIG9mIHRoZSBXQVNNIGJpbmFyeSB1cGxvYWRlZCB0byB0aGUgbmV0d29yayB0aGF0IHdpbGwgYmUgdXNlZCBhcyBhCm5ldyB2ZXJzaW9uIG9mIHRoZSBjb250cmFjdAAAAAAAB3VwZ3JhZGUAAAAAAQAAAAAAAAANbmV3X3dhc21faGFzaAAAAAAAA+4AAAAgAAAAAA==",
        "AAAAAAAAANZVcGdyYWRlcyBhbGwgZGVwbG95ZWQgbWFya2V0IGNvbnRyYWN0cwoKIyMjIEFyZ3VtZW50cwoqIGBuZXdfbWFya2V0X2NvbnRyYWN0X3dhc21faGFzaGAgLSBoYXNoIG9mIHRoZSBXQVNNIGJpbmFyeSB1cGxvYWRlZCB0byB0aGUgbmV0d29yayB0aGF0CndpbGwgYmUgdXNlZCBhcyBhIG5ldyB2ZXJzaW9uIG9mIHRoZSBjb250cmFjdCBmb3IgZXZlcnkgZGVwbG95ZWQgbWFya2V0AAAAAAAYdXBncmFkZV9kZXBsb3llZF9tYXJrZXRzAAAAAQAAAAAAAAAdbmV3X21hcmtldF9jb250cmFjdF93YXNtX2hhc2gAAAAAAAPuAAAAIAAAAAA=",
        "AAAABAAAAB1NYXJrZXQgTWFuYWdlciBDb250cmFjdCBFcnJvcgAAAAAAAAAAAAAITU1DRXJyb3IAAAACAAAAAAAAAA1JbnRlcm5hbEVycm9yAAAAAAAAAAAAAAAAAAATTWFya2V0QWxyZWFkeUV4aXN0cwAAAAAB",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAAAgAAAAAAAAAAAAAABkNvbmZpZwAAAAAAAAAAAAAAAAAKTWFya2V0TGlzdAAA",
        "AAAAAQAAAAAAAAAAAAAABkNvbmZpZwAAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAABltYXJrZXRfY29udHJhY3Rfd2FzbV9oYXNoAAAAAAAD7gAAACA=" ]),
      options
    )
  }
  public readonly fromJSON = {
    deploy: this.txFromJSON<Result<string>>,
        get_market_list: this.txFromJSON<Array<string>>,
        upgrade: this.txFromJSON<null>,
        upgrade_deployed_markets: this.txFromJSON<null>
  }
}