import type {
  i128,
  Option,
  u64,
} from '@stellar/stellar-sdk/contract'
import { Buffer } from 'node:buffer'
import { Address } from '@stellar/stellar-sdk'
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  Spec as ContractSpec,
  MethodOptions,
  Result,
} from '@stellar/stellar-sdk/contract'

export * from '@stellar/stellar-sdk'
export * as contract from '@stellar/stellar-sdk/contract'
export * as rpc from '@stellar/stellar-sdk/rpc'

if (typeof window !== 'undefined') {
  window.Buffer = window.Buffer || Buffer
}

export const LendingContractError = {
  1: { message: 'PoolAlreadyExists' },
  2: { message: 'PoolDoesNotExist' },
  3: { message: 'NonPositiveDeposit' },
  4: { message: 'NonPositiveWithdraw' },
  5: { message: 'ObligationDoesNotExist' },
  6: { message: 'WithdrawOverBalance' },
  7: { message: 'NotEnoughPoolFunds' },
  8: { message: 'OverOrUnderflow' },
  9: { message: 'DepositDoesNotExist' },
  10: { message: 'InvalidLoanPoolConfig' },
  11: { message: 'InvalidLiquidationThreshold' },
  12: { message: 'OracleDoesNotKnowAssetPrice' },
  13: { message: 'HealthFactorIsLowerThanRequiredThreshold' },
}

export interface InterestRates {
  borrow_rate_bps: i128
  supply_rate_bps: i128
}

export interface GlobalState {
  admin: string
  liquidation_threshold_bps: i128
  status: boolean
}

export type DataKey = { tag: 'GlobalState', values: void } | { tag: 'Pool', values: readonly [Address] } | { tag: 'Obligation', values: readonly [Address] } | { tag: 'Accrual', values: void }

export interface Pool {
  borrowed: i128
  config: PoolConfig
  supply: i128
  token_address: string
  token_ticker: string
}

export interface PoolConfig {
  /**
   * Positive Base Rate percentage
   */
  base_rate_bps: i128
  /**
   * Positive Optimal Utilization Ratio percentage
   */
  optimal_utilization_ratio_bps: i128
  /**
   * Non-negative Reserve Ration percentage (< 100)
   */
  reserve_ratio_bps: i128
  slope1: i128
  slope2: i128
}

export interface Obligation {
  borrows: Map<Address, i128>
  deposits: Map<Address, i128>
}

export interface Accrual {
  borrow_accrual: i128
  supply_accrual: i128
  timestamp: u64
}

export interface Client {
  /**
   * Construct and simulate a test_oracle_price transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  test_oracle_price: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean
  }) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a initialize_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize_pool: ({ token_address, token_ticker, salt, pool_config }: { token_address: string, token_ticker: string, salt: Option<Buffer>, pool_config: Option<PoolConfig> }, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean
  }) => Promise<AssembledTransaction<Result<Address>>>

  /**
   * Construct and simulate a deposit transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deposit: ({ user, pool_address, amount }: { user: string, pool_address: string, amount: i128 }, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a borrow transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  borrow: ({ user, pool_address, amount }: { user: string, pool_address: string, amount: i128 }, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a accrue_interest transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  accrue_interest: ({ pool_address }: { pool_address: string }, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a repay transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  repay: ({ pool_address }: { pool_address: string }, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a deposit_collateral transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deposit_collateral: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a withdraw_collateral transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  withdraw_collateral: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a withdraw transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  withdraw: ({ user, pool_address, amount }: { user: string, pool_address: string, amount: i128 }, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_user_obligation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_user_obligation: ({ user }: { user: string }, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean
  }) => Promise<AssembledTransaction<Option<Obligation>>>

  /**
   * Construct and simulate a get_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pool: ({ pool_address }: { pool_address: string }, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean
  }) => Promise<AssembledTransaction<Option<Pool>>>

  /**
   * Construct and simulate a get_interest_rates transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_interest_rates: ({ pool_address }: { pool_address: string }, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean
  }) => Promise<AssembledTransaction<Result<InterestRates>>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
    /** Constructor/Initialization Args for the contract's `__constructor` method */
    { admin, liquidation_threshold }: { admin: string, liquidation_threshold: Option<i128> },
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, 'contractId'> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: 'hex' | 'base64'
      },
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy({ admin, liquidation_threshold }, options)
  }

  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec(['AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAVbGlxdWlkYXRpb25fdGhyZXNob2xkAAAAAAAD6AAAAAsAAAABAAAD6QAAA+0AAAAAAAAH0AAAABRMZW5kaW5nQ29udHJhY3RFcnJvcg==',
        'AAAAAAAAAAAAAAARdGVzdF9vcmFjbGVfcHJpY2UAAAAAAAAAAAAAAQAAAAs=',
        'AAAAAAAAAAAAAAAPaW5pdGlhbGl6ZV9wb29sAAAAAAQAAAAAAAAADXRva2VuX2FkZHJlc3MAAAAAAAATAAAAAAAAAAx0b2tlbl90aWNrZXIAAAARAAAAAAAAAARzYWx0AAAD6AAAA+4AAAAgAAAAAAAAAAtwb29sX2NvbmZpZwAAAAPoAAAH0AAAAApQb29sQ29uZmlnAAAAAAABAAAD6QAAB9AAAAALUG9vbEFkZHJlc3MAAAAH0AAAABRMZW5kaW5nQ29udHJhY3RFcnJvcg==',
        'AAAAAAAAAAAAAAAHZGVwb3NpdAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAUTGVuZGluZ0NvbnRyYWN0RXJyb3I=',
        'AAAAAAAAAAAAAAAGYm9ycm93AAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAUTGVuZGluZ0NvbnRyYWN0RXJyb3I=',
        'AAAAAAAAAAAAAAAPYWNjcnVlX2ludGVyZXN0AAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAABRMZW5kaW5nQ29udHJhY3RFcnJvcg==',
        'AAAAAAAAAAAAAAAFcmVwYXkAAAAAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAUTGVuZGluZ0NvbnRyYWN0RXJyb3I=',
        'AAAAAAAAAAAAAAASZGVwb3NpdF9jb2xsYXRlcmFsAAAAAAAAAAAAAA==',
        'AAAAAAAAAAAAAAATd2l0aGRyYXdfY29sbGF0ZXJhbAAAAAAAAAAAAA==',
        'AAAAAAAAAAAAAAAId2l0aGRyYXcAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAUTGVuZGluZ0NvbnRyYWN0RXJyb3I=',
        'AAAAAAAAAAAAAAATZ2V0X3VzZXJfb2JsaWdhdGlvbgAAAAABAAAAAAAAAAR1c2VyAAAAEwAAAAEAAAPoAAAH0AAAAApPYmxpZ2F0aW9uAAA=',
        'AAAAAAAAAAAAAAAIZ2V0X3Bvb2wAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAA+gAAAfQAAAABFBvb2w=',
        'AAAAAAAAAAAAAAASZ2V0X2ludGVyZXN0X3JhdGVzAAAAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAA+kAAAfQAAAADUludGVyZXN0UmF0ZXMAAAAAAAfQAAAAFExlbmRpbmdDb250cmFjdEVycm9y',
        'AAAABAAAAAAAAAAAAAAAFExlbmRpbmdDb250cmFjdEVycm9yAAAADQAAAAAAAAARUG9vbEFscmVhZHlFeGlzdHMAAAAAAAABAAAAAAAAABBQb29sRG9lc05vdEV4aXN0AAAAAgAAAAAAAAASTm9uUG9zaXRpdmVEZXBvc2l0AAAAAAADAAAAAAAAABNOb25Qb3NpdGl2ZVdpdGhkcmF3AAAAAAQAAAAAAAAAFk9ibGlnYXRpb25Eb2VzTm90RXhpc3QAAAAAAAUAAAAAAAAAE1dpdGhkcmF3T3ZlckJhbGFuY2UAAAAABgAAAAAAAAASTm90RW5vdWdoUG9vbEZ1bmRzAAAAAAAHAAAAAAAAAA9PdmVyT3JVbmRlcmZsb3cAAAAACAAAAAAAAAATRGVwb3NpdERvZXNOb3RFeGlzdAAAAAAJAAAAAAAAABVJbnZhbGlkTG9hblBvb2xDb25maWcAAAAAAAAKAAAAAAAAABtJbnZhbGlkTGlxdWlkYXRpb25UaHJlc2hvbGQAAAAACwAAAAAAAAAbT3JhY2xlRG9lc05vdEtub3dBc3NldFByaWNlAAAAAAwAAAAAAAAAKEhlYWx0aEZhY3RvcklzTG93ZXJUaGFuUmVxdWlyZWRUaHJlc2hvbGQAAAAN',
        'AAAAAQAAAAAAAAAAAAAADUludGVyZXN0UmF0ZXMAAAAAAAACAAAAAAAAAA9ib3Jyb3dfcmF0ZV9icHMAAAAACwAAAAAAAAAPc3VwcGx5X3JhdGVfYnBzAAAAAAs=',
        'AAAAAQAAAAAAAAAAAAAAC0dsb2JhbFN0YXRlAAAAAAMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAZbGlxdWlkYXRpb25fdGhyZXNob2xkX2JwcwAAAAAAAAsAAAAAAAAABnN0YXR1cwAAAAAAAQ==',
        'AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABAAAAAAAAAAAAAAAC0dsb2JhbFN0YXRlAAAAAAEAAAAAAAAABFBvb2wAAAABAAAH0AAAAAtQb29sQWRkcmVzcwAAAAABAAAAAAAAAApPYmxpZ2F0aW9uAAAAAAABAAAH0AAAAAtVc2VyQWRkcmVzcwAAAAAAAAAAAAAAAAdBY2NydWFsAA==',
        'AAAAAQAAAAAAAAAAAAAABFBvb2wAAAAFAAAAAAAAAAhib3Jyb3dlZAAAAAsAAAAAAAAABmNvbmZpZwAAAAAH0AAAAApQb29sQ29uZmlnAAAAAAAAAAAABnN1cHBseQAAAAAACwAAAAAAAAANdG9rZW5fYWRkcmVzcwAAAAAAABMAAAAAAAAADHRva2VuX3RpY2tlcgAAABE=',
        'AAAAAQAAAAAAAAAAAAAAClBvb2xDb25maWcAAAAAAAUAAAAdUG9zaXRpdmUgQmFzZSBSYXRlIHBlcmNlbnRhZ2UAAAAAAAANYmFzZV9yYXRlX2JwcwAAAAAAAAsAAAAtUG9zaXRpdmUgT3B0aW1hbCBVdGlsaXphdGlvbiBSYXRpbyBwZXJjZW50YWdlAAAAAAAAHW9wdGltYWxfdXRpbGl6YXRpb25fcmF0aW9fYnBzAAAAAAAACwAAAC5Ob24tbmVnYXRpdmUgUmVzZXJ2ZSBSYXRpb24gcGVyY2VudGFnZSAoPCAxMDApAAAAAAARcmVzZXJ2ZV9yYXRpb19icHMAAAAAAAALAAAAAAAAAAZzbG9wZTEAAAAAAAsAAAAAAAAABnNsb3BlMgAAAAAACw==',
        'AAAAAQAAAAAAAAAAAAAACk9ibGlnYXRpb24AAAAAAAIAAAAAAAAAB2JvcnJvd3MAAAAD7AAAB9AAAAALUG9vbEFkZHJlc3MAAAAACwAAAAAAAAAIZGVwb3NpdHMAAAPsAAAH0AAAAAtQb29sQWRkcmVzcwAAAAAL',
        'AAAAAQAAAAAAAAAAAAAAB0FjY3J1YWwAAAAAAwAAAAAAAAAOYm9ycm93X2FjY3J1YWwAAAAAAAsAAAAAAAAADnN1cHBseV9hY2NydWFsAAAAAAALAAAAAAAAAAl0aW1lc3RhbXAAAAAAAAAG']),
      options,
    )
  }

  public readonly fromJSON = {
    test_oracle_price: this.txFromJSON<i128>,
    initialize_pool: this.txFromJSON<Result<Address>>,
    deposit: this.txFromJSON<Result<void>>,
    borrow: this.txFromJSON<Result<void>>,
    accrue_interest: this.txFromJSON<Result<void>>,
    repay: this.txFromJSON<Result<void>>,
    deposit_collateral: this.txFromJSON<null>,
    withdraw_collateral: this.txFromJSON<null>,
    withdraw: this.txFromJSON<Result<void>>,
    get_user_obligation: this.txFromJSON<Option<Obligation>>,
    get_pool: this.txFromJSON<Option<Pool>>,
    get_interest_rates: this.txFromJSON<Result<InterestRates>>,
  }
}
