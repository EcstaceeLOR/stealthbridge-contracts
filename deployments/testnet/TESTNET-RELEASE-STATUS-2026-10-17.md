# Soroban Testnet release readiness — 17 October 2026

> **State: NOT DEPLOYED — no signed transaction or on-chain contract ID has been verified.**
> This page records reproducible *build evidence* and an incomplete deployment
> preflight; it is not a deployed-contract announcement.

## Reviewed CI release

- **Source commit:** [`130c577ef6f44412a249d43e3e459cedecb5c970`](https://github.com/stealthbridge-labs/stealthbridge-contracts/commit/130c577ef6f44412a249d43e3e459cedecb5c970)
- **Successful workflow:** [Soroban contract checks](https://github.com/stealthbridge-labs/stealthbridge-contracts/actions/runs/39030590122)
- **Workflow artifact:** `stealthbridge-offline-testnet-kit-130c577ef6f44412a249d43e3e459cedecb5c970` (GitHub Actions artifact ID `12100168086`)
- **Toolchain:** Rust 1.91.0 · Soroban SDK 27.0.6 · Stellar CLI 27.1.0 · `wasm32v1-none`.
- The artifact kit's `SHA256SUMS`, `provenance.json`, three WASM binaries and three ABI files passed `scripts/verify_artifacts.py` using the exact archive from the successful CI run. Each WASM hash matches both isolated builds recorded in provenance.

| Soroban source contract | CI WASM SHA-256 | CI ABI SHA-256 | Testnet contract ID |
| --- | --- | --- | --- |
| `corridor-registry` | `588d17f99850272f2f18e6ee407d94329b4f8ae415c2f6b71993cf59f4139099` | `727ab656c23f1dd52fbcd194b5bbb184e0627c7e2d94d77f2e7945c11d76bec1` | **Not deployed / not verified** |
| `policy-registry` | `856186ba3dca1feff010c670bd1ae9818bf69e284304273f943745603b9a7912` | `5a21c448c76903b26116807b181c92bb8cbc2882e85c410aac301e5c7a72e072` | **Not deployed / not verified** |
| `governance-gate` | `66b40fe5ee4f704a91acf39922ed130cc1eb4fc0cdc3c5ee5a30614921f4aa81` | `917f0d13c5373d7f33aa56e8e1760025337a791166e83fec788a0c27a2e5b3e1` | **Not deployed / not verified** |

**Artifact hashes describe locally compiled code.** They are not deployment
transaction hashes, network-upload hashes, or proofs of live contracts.

## Preflight observation (not deployment)

- `scripts/preflight_testnet.py` passed with three checked artifacts, an empty
  `not-deployed` manifest and a checksum-valid proposed public administrator.
- The public administrator account was found through the **Stellar Testnet
  Horizon** read API on 17 October 2026, with **10,000 Testnet XLM** at the
  moment of the check. This volatile observation does not prove ownership,
  authority to sign or future balance.
- Stellar CLI 27.1.0 was installed from the official GitHub v27.1.0 release
  and the downloaded x86_64 archive SHA-256 checked against published
  release metadata.
- The workstation had **no configured Stellar CLI identities**, and
  `stellar keys address operator_local` reported that no such identity exists.
  No seed phrase, signing key or wallet approval was accessed or created.
- The workstation's direct public Soroban RPC `getNetwork` and
  `getLatestLedger` calls returned HTTP 403. Therefore the live network
  passphrase, current protocol, ledger freshness and bytecode state were
  **not established by this preflight**.
- **No uploads, deploy calls, signed transactions, or payment operations
  were performed.** No new `C...` IDs or transaction hashes exist in this
  release evidence.

## Remaining operator and attestation gates

1. Confirm the approved Testnet administrator **controls** an authorized
   local or hardware-backed signer (do not upload secrets to the repository,
   CI, chat, or Vercel); the CLI account must resolve to the intended public
   administrator.
2. Complete live RPC network/passphrase/protocol/freshness checks with an
   approved reachable Testnet endpoint; recheck the account balance.
3. Review the exact pinned WASM and constructor parameters; obtain explicit
   authorization for signing. Deployment dependency order is
   `corridor-registry` → `policy-registry` → `governance-gate`.
   The gate must bind the **actual verified** C-addresses of the first two.
4. Save **real Testnet deployment transaction IDs**, C-addresses, operator
   approvals, ledgers, independently read network state, fetched on-chain
   WASM hashes, admin/paused state, immutable gate dependencies and TTL.
   Run [the on-chain WASM verifier](../../scripts/verify_onchain_wasm.py).
5. Implement and independently review a **three-contract deployment
   attestation schema**, including constructor bindings, before changing
   the canonical [v1 manifest](manifest.json). Version 1 cannot attest
   all three deployments. Synchronize a reviewed release with backend, SDK
   and frontend without implying any payment capability.

**Do not modify** `manifest.json` to `deployed` based on this report.
The [operator runbook](OPERATOR-DEPLOYMENT-RUNBOOK.md), the
[deployment evidence gates](evidence-chain.v1.json) and the
[release planner](../../scripts/plan_testnet_release.py) remain authoritative
for the signing and subsequent independent review.
Financial execution, fiat settlement and privacy-verification flags stay
**disabled**, regardless of governance-contract deployment.
