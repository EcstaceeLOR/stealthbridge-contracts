# Public governance code health — Stellar Testnet

The three **governance** contracts are deployed on Testnet. Their original
bytecode was verified at deployment. This document covers the **new separate**
current-state read path; neither observation is authorization to transfer funds.

## Three distinct trust levels

| Check | Source | What it proves | What it does not prove |
| --- | --- | --- | --- |
| Deployment receipt | Public Testnet Horizon and pinned release evidence | Original deployment transaction succeeded at the recorded ledger | Current administrator, code hash, privacy, fees |
| Current contract instance | Stellar RPC `getLedgerEntries`, strict base64 XDR prefix decoding | At a recently observed ledger the exact C-address points to a particular WASM *executable hash* and has a recorded TTL | Actual contents of the uploaded WASM, ownership/admin state, payment readiness |
| Full current bytecode attestation | `stellar contract fetch` + reproducible pinned WASM files | Fetched current WASM bytes match reviewed artifacts | Authority to spend customer or operator funds, confidential settlement verification |

The Rust backend now separates `GET /v1/governance/release` (historical
receipts) from `GET /v1/governance/code` (current contract instances). The
TypeScript SDK exposes `governanceRelease()` and `governanceCode()` with
strict response validation. The frontend technical preview displays both
when independent public Testnet checks succeed. The public website at
`/protocol` remains a **historical** static release record, not a current
network-health assertion.

## Current instance hash observation

1. Verify the network passphrase is exactly
   `Test SDF Network ; September 2015`; reject another network.
2. Require a recent, bounded RPC ledger close time.
3. Query all three known contract-instance LedgerKey XDR values with
   `getLedgerEntries` using `xdrFormat=base64`, no signing or wallet.
4. Decode the fixed `LedgerEntryData.ContractData` instance prefix and
   compare the actual contract address, executable hash, ledger sequence and
   live-until sequence. Do not interpret or publish arbitrary contract instance
   storage, which might include private or tenant-linked records.
5. Compare the hash against the **specific October 17, 2026 reviewed
   release artifact hash**. A mismatch can be an authorized upgrade OR a
   dangerous drift: it is not a blanket failure of Stellar, but requires
   explicit new-code review.
6. When an instance is missing, expired, malformed, from another network or
   inconsistent with the observed head, **return unavailable**. Never replace
   the result with static evidence or label the contract healthy.

The current hash parser reads only the XDR header and the executable hash.
It is intentionally **not a full XDR parser**, current bytecode readback, or
administrator identity verification. Check actual WASM with the existing
`scripts/verify_onchain_wasm.py` CLI-only workflow and pinned artifacts.

## Current storage / current administrator read

The same fresh Testnet `getLedgerEntries` response now also supports
`GET /v1/governance/admin`: the backend decodes each **instance SCMap** and
reads the exact `Admin` and `PendingAdmin` enum keys, registry
`Paused` flags, and immutable GovernanceGate registry bindings. It refuses
unrecognized bytecode, malformed storage and duplicate or noncanonical enum
keys. Admin address observations are **not proof that anyone still controls
the private key**. The TypeScript SDK refuses stale, future-dated or missing
ledger-close timestamps; the frontend reports pending successor nominations
without taking custody or requesting signatures.

**XDR distinction that must remain pinned in tests:** `LedgerKeyContractData`
encodes `CONTRACT_DATA(6) + SCAddress.Contract(1) + contract ID +
SCV_LEDGER_KEY_CONTRACT_INSTANCE(20) + PERSISTENT(1)`. The corresponding
`LedgerEntryData.ContractData` encodes `CONTRACT_DATA(6) +
ContractDataEntry.ext(0) + SCAddress.Contract(1) + contract ID + key +
durability + SCV_CONTRACT_INSTANCE(19) + ContractExecutable.Wasm(0) +
hash + SCMap*`. Omitting **only the entry extension** can make offline
fixtures appear valid while actual Stellar RPC responses fail. Backend unit
tests include the public Stellar documentation's real XDR example to guard
this distinction.

Official reference: https://developers.stellar.org/docs/data/apis/rpc/api-reference/methods/getLedgerEntries

## Administrator continuity and fee-claim recovery

The original administrator and deployer is
`GC3YRVLB7IN7SNUYVWFUC6OD3QWJXCUZ4HOBNIPVGY7BZDVOYMDNX5TN`.
Its local CLI alias `operator_local` is not the chain identity: **the public
key is**. A new PC with a newly generated key called `operator_local` is
not automatically an administrator, even if funded.

Before losing the Fedora secure-store signer, use the contracts' reviewed
two-step administrator handover with an independently recoverable and safely
backed-up successor; inspect the current contract ABI and Testnet state at
execution time. Do not paste, commit, export to support staff, or transmit
the secret seed. The separate fee vault has the same operational consideration:
the future `claim_fees` authorization is bound to the current vault admin;
the fee vault is still **not deployed or audited**.

For Mainnet, independent contract upgrades, payments, fee collection,
business identity, multisig/hardware signing, custody, reconciliation, and
financial/compliance review remain separate release gates. No current read-only
endpoint changes the policy flags: privacy=false, transfers=false,
fee_collection=false.
