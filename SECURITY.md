# Security policy

## Reporting a vulnerability

Report vulnerabilities privately, never in a public issue or pull request:

- open a [private vulnerability report](https://github.com/pointgroup-labs/alula/security/advisories/new) on GitHub, or
- email [support@alula.finance](mailto:support@alula.finance).

Name the affected contract or package, the commit or deployed contract address, and the steps that reproduce the issue. A failing test or a transaction simulation helps most.

## Scope

- Contracts in `contracts/` and their interfaces in `libs/`.
- Contracts deployed on Stellar from this repository's tagged releases; each release attests its WASM build.
- The TypeScript SDK and CLI in `packages/`.

## Audits

Halborn and Highland Security reviewed the contracts; their reports are in [`audits/`](audits/).
