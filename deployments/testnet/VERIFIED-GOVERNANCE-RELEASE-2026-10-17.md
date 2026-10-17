# Verified Soroban governance deployment — Stellar Testnet (17 October 2026)

**Three governance contracts are deployed on Stellar Testnet.** This is a public
governance-only Testnet release, **not** a confidential transfer, wallet bridge,
asset, fiat payout, or platform-fee collection release.

## Provenance and accounts

- **Reviewed source:** [`130c577ef6f44412a249d43e3e459cedecb5c970`](https://github.com/stealthbridge-labs/stealthbridge-contracts/commit/130c577ef6f44412a249d43e3e459cedecb5c970)
- **Build evidence:** [successful Soroban CI, 39030590122](https://github.com/stealthbridge-labs/stealthbridge-contracts/actions/runs/39030590122) and 3/3 reproducible WASM + ABI hashes
- **Runtime:** Stellar Testnet; passphrase `Test SDF Network ; September 2015`; observed protocol 29
- **Deployment signer and on-chain administrator:** `GC3YRVLB7IN7SNUYVWFUC6OD3QWJXCUZ4HOBNIPVGY7BZDVOYMDNX5TN` (`operator_local` CLI alias; OS secure-store identity)
- **Previous Freighter wallet:** `GBBCOFZZ3RDQHTC662EVM3UO5SAXQEZB6DL64HLEJYEYJ4KBVL3CDLTE` is **not** administrator of this release and did not sign these deployments
- **Machine-readable observed release:** [`governance-deployment-evidence.v2.json`](governance-deployment-evidence.v2.json)

## Real deployed contract identities

| Contract | Stellar Testnet contract ID | Deployment tx / ledger |
| --- | --- | --- |
| **CorridorRegistry** | `CDS6HBE6YJSZTRG3VR2HLCNHWJZMKB43BLFFEHEH3Z76WXAF46LW7JOB` | [`0ac409…43e5`](https://stellar.expert/explorer/testnet/tx/0ac4094247fd50872d079c9637b0975f825f99574e6722c9c69739d8c5c143e5), 5244269 |
| **PolicyRegistry** | `CBXWEA5PMDTBN5AN7FJYGGAPIWLRKXYHWLVWG5WOXH3HSRDB4RYTJBN5` | [`34c2e8…e118`](https://stellar.expert/explorer/testnet/tx/34c2e84cad5be5181b7e1964d4e48e2c26ca021583373922f821c0038abce118), 5244283 |
| **GovernanceGate** | `CD2KIMDRYJMKGO5SMIBOQP6ADPW5I4THNZFGXWYLIPGSXAI5NRXVAPKV` | [`946880…341`](https://stellar.expert/explorer/testnet/tx/9468801411a6bbb7fb2966c0225057bb4f3c8a7113aa687c0a0be744a7c5b341), 5244293 |

Actual WASM upload receipts: CorridorRegistry [`438313…5d48`](https://stellar.expert/explorer/testnet/tx/43831300c653b110b83a8d5074c33337bb98819ef83d889e727eab7b96965d48); PolicyRegistry [`b6b447…e428`](https://stellar.expert/explorer/testnet/tx/b6b447f587be6d4e0446f5666165cb8b947be415522e76bc61d32626f0c9e428); GovernanceGate [`93d062…7802`](https://stellar.expert/explorer/testnet/tx/93d0625fa98862a76ae9156400eb0eba16a9d57a8a079301b94acb6f0da37802).

All six public Testnet upload/deployment transactions (including the earlier CorridorRegistry WASM upload) were confirmed **successful** through Horizon. This is a public transaction-state check, not an independent security audit.

## What was read back from Testnet

| Contract | Uploaded WASM SHA-256, independently fetched from deployed contract | On-chain state read |
| --- | --- | --- |
| CorridorRegistry | `588d17f99850272f2f18e6ee407d94329b4f8ae415c2f6b71993cf59f4139099` | `get_admin` = GC3Y…NX5TN; `is_paused` = false |
| PolicyRegistry | `856186ba3dca1feff010c670bd1ae9818bf69e284304273f943745603b9a7912` | `admin` = GC3Y…NX5TN; `is_paused` = false |
| GovernanceGate | `66b40fe5ee4f704a91acf39922ed130cc1eb4fc0cdc3c5ee5a30614921f4aa81` | `get_admin` = GC3Y…NX5TN; `corridor_registry` and `policy_registry` exactly match the two deployed IDs above |

The exact deployed WASM bytes were fetched with `stellar contract fetch --id ... --network testnet` and matched the pinned artifacts byte-for-byte. `scripts/verify_onchain_wasm.py` can repeat the three comparisons using the IDs in the evidence JSON. Read-only on-chain state calls used `stellar contract invoke` and the Testnet network passphrase.

## Manifest, SDK and product release boundary

The historical `deployments/testnet/manifest.json` is a **legacy v1 shape** whose
validator only supports the original single-contract deployment fixture. Its
`not-deployed` value is **not a statement about today's observed Testnet ledger**;
it remains unmodified to avoid publishing an incomplete canonical three-contract
attestation to older backend/SDK/frontend consumers.

The new v2 **observed deployment** record is a separately validated technical
receipt, **not yet** a wire-compatible replacement for the existing canonical
manifest. Its offline validator checks schema, all three StrKey checksums,
constructor bindings, source/WASM/ABI digests, and disables all financial claims.
It cannot certify live chain freshness by itself. A reviewed canonical v2
migration and backend/SDK/frontend RPC-backed read adapter are still needed.

**Financial boundaries:** no privacy proof is verified; no token/fiat settlement,
fee collection, custody, or payout is enabled. Real Testnet governance flags are
not evidence of executable payment rails. Platform-fee quoting can be developed
separately as a pure SDK calculation, with transparent opt-in consent and a
later independently reviewed fee-collecting contract.

## Developer verification

```sh
# Extract the pinned artifact archive from successful CI run 39030590122,
# source commit 130c577e, into a directory on your own computer.
python3 scripts/verify_artifacts.py --artifacts PATH_TO_PINNED_ARTIFACTS
python3 scripts/validate_governance_observation.py --artifacts PATH_TO_PINNED_ARTIFACTS
# Read-only: use actual contract IDs and the exact same release artifacts.
python3 scripts/verify_onchain_wasm.py --contracts-file YOUR_THREE_CONTRACT_IDS.json --artifacts PATH_TO_PINNED_ARTIFACTS
```

The offline validator intentionally requires the **original 130c577e release artifacts**. Building current HEAD after documentation-only commits produces new source provenance and is not a substitute for the reviewed deployment release.

Use a local JSON object mapping keys `corridor-registry`, `policy-registry`,
and `governance-gate` to the real contract IDs above for the last command.
**Do not** expose secret keys or a seed phrase when checking these IDs.
