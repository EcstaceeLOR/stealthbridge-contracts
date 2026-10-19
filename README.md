<div align="center"><img src="assets/stealthbridge-logo.svg" width="760" alt="StealthBridge — Confidential payments. Without borders." /></div>

# StealthBridge Protocol & Smart Contracts

## How deployed contracts connect to StealthBridge

**Observed on Stellar Testnet (17 October 2026):** `CorridorRegistry`, `PolicyRegistry` and `GovernanceGate` are actual Soroban/Rust→WASM deployed governance instances. Exact on-chain IDs, receipts and hashes: [verified governance release](deployments/testnet/VERIFIED-GOVERNANCE-RELEASE-2026-10-17.md) · [machine-readable deployment observation](deployments/testnet/governance-deployment-evidence.v2.json). They manage **public governance state** and cross-registry decisions, not funds.

```text
Next.js product / preview --GET /api/bridge/*--> Rust Axum backend
          |                                    |-- Stellar Testnet RPC
          |                                    |     |-- CorridorRegistry
          |                                    |     |-- PolicyRegistry
          |                                    |     '-- GovernanceGate
          |                                    '-- Neon PostgreSQL (journal/catalog)
          '-- read-only browser wallet context
TypeScript SDK -------- typed, validated API data --------^
```

The backend's `/v1/governance/release`, `/v1/governance/code` and `/v1/governance/admin` reads underpin the frontend's gated `/preview/governance` experience. The frontend does **not** submit Soroban invocations or move assets. A fresh executable/admin observation is separate from cryptographic authorization or privacy proofs. See the [cross-repository topology](https://github.com/stealthbridge-labs/.github/blob/main/docs/INTEGRATION-TOPOLOGY.md) and [attestation handoff](docs/GOVERNANCE-ATTESTATION-HANDOFF.md). Canonical `manifest.json` v1 remains an outdated `not-deployed` record until reviewed v2 promotion. **Platform fees, fund transfers, confidential rails and fiat payouts are not operational.**

**Engineering roadmap:** [View the repository-specific plan](ROADMAP.md).

## October 17 — verified on-chain Testnet governance deployment

**All three Soroban governance contracts are deployed and their on-chain WASM
verified against pinned CI artifacts.** The deployed administrator is the
`operator_local` Stellar CLI account (`GC3Y…NX5TN`), **not** the separate
`GBBC…DLTE` Freighter account. Their exact C-addresses, public transaction
hashes, constructor bindings and code hashes are recorded in the
[verified governance release report](deployments/testnet/VERIFIED-GOVERNANCE-RELEASE-2026-10-17.md)
and [machine-readable evidence](deployments/testnet/governance-deployment-evidence.v2.json).

The **legacy canonical manifest v1** still reads `not-deployed` because
its validator cannot represent all three contracts. This now describes an
outdated integration record, **not the live chain**; a reviewed manifest v2
migration is needed before backend/SDK/frontend can consume these addresses
as trusted live contracts. No private payments, platform fee collection,
custody, or fiat payout have been enabled.

## Rust-first Soroban engineering

**All on-chain StealthBridge smart contracts are written in Rust and compiled to WASM.**
The Python scripts under `scripts/` are off-chain maintainers' build, provenance,
deployment-preflight, and test utilities—not a Python payment protocol.
Counting their individual test files can make GitHub's language breakdown
look Python-heavy even though Soroban validators execute compiled Rust WASM.

The [Rust-first architecture and Python-tooling plan](docs/RUST-FIRST-SOROBAN.md)
explains the exact boundaries and how we are progressively moving source
audits and selected release tooling to native Rust without losing digest
verification. CI now compiles and exercises `tools/source-audit.rs` with
`rustc` alongside all three Soroban crates.

**New actual on-chain Rust methods:** `CorridorRegistry::approve_config` and `is_enabled_with_digest` bind an already-enabled corridor to a revocable public digest and bounded ledger expiry; `PolicyRegistry::is_effective_commitment`
binds an approved revision and `BytesN<32>` public commitment;
`GovernanceGate::public_flags_allow_commitment` checks the bound policy
and corridor through Soroban cross-contract invocations; and
`GovernanceGate::check_commitment_batch` processes at most eight
ordered public governance decisions with an explicit oversize error.
These do **not** prove privacy, transfer funds, or authorize fiat payout.
## Soroban architecture: three contracts and the route to Testnet

**Full engineering guide:** [Soroban architecture, source-to-WASM provenance, wallet signing boundaries, and delivery milestones](docs/ARCHITECTURE-AND-DELIVERY.md).

```text
CorridorRegistry ── public is_enabled(corridor) ─┐
                                                 ├── GovernanceGate
PolicyRegistry ── public is_effective(policy) ───┘    │
                                                      └─ public_flags_allow()
Three verified Testnet governance deployments → pending backend safe-read adapter
Future audited private rail and settlement adapter (NOT implemented)
```

**What exists:** three actual Rust/Soroban crates—[corridor registry](contracts/corridor-registry/README.md), [policy registry](contracts/policy-registry/README.md), and [read-only governance gate](contracts/governance-gate/README.md). They have scoped authorization, emergency-pause logic, local cross-contract tests and reproducible WASM/ABI artifact evidence. A combined public governance flag does not authorize token transfer, validate a privacy proof, or confirm settlement eligibility.

**Current deployment state:** the three actual contract IDs and transactions are published in the [verified Testnet release evidence](deployments/testnet/VERIFIED-GOVERNANCE-RELEASE-2026-10-17.md). The old manifest v1 still says `not-deployed` and requires a reviewed three-contract compatibility migration. No privacy verifier, issuer-backed asset, custody, fee collector, or fiat payout is verified.

**Near-term integration plan:** migrate the canonical manifest to an independently reviewed three-contract format, sync typed ABI hashes across backend and SDK, maintain and harden the existing read-only Testnet RPC verification of the three contract IDs and frontend governance workspace without enabling any payment or privacy rail. Fee charging requires a separate audited asset-transfer contract and transparent user consent.

**Privacy roadmap:** evaluate Confidential Tokens for value confidentiality and Stellar Private Payments for relationship privacy as separate systems. Define issuer authority, private proof inputs, nullifier/replay protections, fee/TTL, metadata disclosure, account recovery and regulator access. Implement fund-moving contracts **only** when the chosen primitive, legal model, asset issuer and external settlement arrangements have passed security review.

**Verification commands:** `cargo test --workspace --locked`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `python3 scripts/artifacts.py`, `python3 scripts/verify_artifacts.py`, and `python3 scripts/preflight_testnet.py`. A local WASM match is not evidence that anything has been deployed.

**Confidential payments. Without borders.**

[Frontend](https://github.com/stealthbridge-labs/stealthbridge-frontend) · [Backend](https://github.com/stealthbridge-labs/stealthbridge-backend) · [SDK](https://github.com/stealthbridge-labs/stealthbridge-sdk)

> **Status:** research-stage Stellar **Testnet-only** protocol. Three public governance Soroban contracts are **deployed and their bytecode/admin/dependencies verified on Testnet**. No privacy proofs, platform fee collection, or fund-moving settlement primitives are operational.

## What is here?
- `contracts/corridor-registry`: authorized and pausable on-chain public corridor governance.
- `contracts/policy-registry`: revisioned public commitments and strict commitment matching.
- `contracts/governance-gate`: fail-closed cross-registry and bounded batch reads in Rust.
- \`docs/PROTOCOL-ARCHITECTURE.md\`: confidentiality vs anonymity, issuer-controlled stablecoins, planned boundaries.
- \`docs/TESTPLAN.md\`: formal verification milestones, no credentials required.
- \`deployments/testnet/manifest.json\`: legacy v1 integration manifest, pending reviewed replacement; see [actual three-contract Testnet evidence](deployments/testnet/governance-deployment-evidence.v2.json).
- \`docs/PRIVACY-MODEL.md\`: initial threat model.
- \`docs/REFERENCES.md\`: official developer resources.

## Build & test
The repository pins Rust **1.91.0**, Soroban SDK **27.0.6**, Stellar CLI
**27.1.0**, and the `wasm32v1-none` target (protocol **27** baseline).
Install the CLI from its [official release](https://github.com/stellar/stellar-cli/releases/tag/v27.1.0).
Rustup reads `rust-toolchain.toml` automatically.

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --locked --target wasm32v1-none --release
rustc --edition=2021 --test -D warnings tools/source-audit.rs -o /tmp/soroban-audit-tests
/tmp/soroban-audit-tests
python3 -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/artifacts.py
python3 scripts/verify_artifacts.py
python3 scripts/validate_manifest.py
python3 scripts/preflight_testnet.py
python3 scripts/benchmark.py
```

These commands run locally without signer credentials. CI publishes the WASM,
ABI snapshot, source/tool versions, checksums, and a quantitative resource table.
The artifact script compares two isolated release builds byte-for-byte.
[Verification and benchmark details](docs/REGISTRY-VERIFICATION.md) describe
what is measured and the local test limitations. The legacy canonical manifest remains `not-deployed` despite this now-verified
governance Testnet release. Build reproducibility alone does not prove deployment;
the [chain readback evidence](deployments/testnet/VERIFIED-GOVERNANCE-RELEASE-2026-10-17.md)
separately records successful on-chain transactions, WASM, admin and dependency checks.
This is not settlement or privacy verification.

The CI pipeline runs the current workspace tests, Clippy, reproducible
WASM builds of all three contracts, per-contract ABI hashing, artifact-integrity
verification, manifest checks and an offline deployment preflight. Historic
macOS test counts and WASM sizes are not a current release baseline; use
pinned CI artifacts and `artifacts/provenance.json` for measurements. Local
Soroban-host tests do **not** verify actual signatures, network restoration
fees, deployed contracts or payment execution.

## Architecture
StealthBridge Business targets **confidential amounts with known parties**; StealthBridge Send targets **shielded relationships**. These require separate privacy primitives with different trust and metadata leakage characteristics. The governance registries and gate remain intentionally independent of both privacy rails until real Testnet composability is proven.

## Prior-art and diligence
Tukar provides a compelling testnet reference for private remittances, proofs and compliance. StealthBridge's proposed differentiators are multi-provider tenancy, modular settlement adapters, B2B confidentiality, stablecoin issuer controls, and public SDK surfaces. These are planned capabilities, not shipped claims.

## Legacy scaffolding
This repo was renamed from the original monorepo. Historical \`apps/\`, \`services/\`, and \`packages/\` placeholders remain until reviewed; **implementation now belongs to the dedicated repositories above**. We preserve content rather than destroying it.

## Safety
No real funds; never commit credentials or witnesses. Review [SECURITY.md](SECURITY.md). License selection and contributor governance will be settled through open development.

## PolicyRegistry — separate governance prototype

The workspace includes the PolicyRegistry alongside the CorridorRegistry and the read-only GovernanceGate: [PolicyRegistry](contracts/policy-registry/README.md). It offers admin-authenticated policy commitment updates, strictly increasing revisions, and a fail-closed global pause. The governance contracts are intentionally **public configuration primitives**, not custody or ZK settlement solutions.

Refer to [the policy security model](docs/POLICY-REGISTRY.md) and [the comprehensive roadmap](ROADMAP.md). The governance contract IDs are attested on Testnet; no live asset integration or privacy rail is claimed.

## Detailed implementation guide

[Contract implementation guide](docs/IMPLEMENTATION-GUIDE.md) documents the evolving Soroban workspace, authorization rules, verification procedures, TTL handling and deployment boundaries.

## Source-level contract interface shared with the SDK

[`integrations/public-soroban-interface.v1.json`](integrations/public-soroban-interface.v1.json) inventories the *actual* CorridorRegistry, PolicyRegistry and GovernanceGate read methods and separates them from administrator-controlled writes. CI checks the public method names against Rust/Soroban source before an SDK can mirror the interface. This is a source ABI reference, **not** evidence of an on-chain contract instance, a privacy circuit, or a live transfer.

## Customer wallet and privacy delivery

Product integrations for Freighter, programmable passkey accounts, Confidential Tokens and Private Payments are tracked in [Wallet, Smart Account and ZK integration](docs/WALLET-SMART-ACCOUNT-AND-PRIVACY-INTEGRATION.md). The real public wallet connection is live on the frontend; smart-wallet deployment and verified private transfers are not yet enabled.
