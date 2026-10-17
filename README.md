<div align="center"><img src="assets/stealthbridge-logo.svg" width="760" alt="StealthBridge — Confidential payments. Without borders." /></div>

# StealthBridge Protocol & Smart Contracts

**Engineering roadmap:** [View the repository-specific plan](ROADMAP.md).

## October 17 Testnet release verification

The latest [Soroban CI build at `130c577e`](https://github.com/stealthbridge-labs/stealthbridge-contracts/actions/runs/39030590122) reproduced and verified the three governance WASM/ABI artifacts. The official pinned Stellar CLI 27.1.0 is installed in the operator workstation, and the proposed administrator account exists on Stellar Testnet. **On-chain deployment is still pending:** the workstation has no matching CLI signer, a direct public RPC preflight returned HTTP 403, and there are **no attested contract C-addresses or signed deployment transactions**. The canonical manifest stays `not-deployed` and payments remain disabled.

See the [October 17 release evidence and remaining deployment gates](deployments/testnet/TESTNET-RELEASE-STATUS-2026-10-17.md). Those exact CI WASM hashes are *build provenance*, not deployed contract IDs.


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
Future independently attested Testnet deployments → backend safe reads
Future audited private rail and settlement adapter (NOT implemented)
```

**What exists:** three actual Rust/Soroban crates—[corridor registry](contracts/corridor-registry/README.md), [policy registry](contracts/policy-registry/README.md), and [read-only governance gate](contracts/governance-gate/README.md). They have scoped authorization, emergency-pause logic, local cross-contract tests and reproducible WASM/ABI artifact evidence. A combined public governance flag does not authorize token transfer, validate a privacy proof, or confirm settlement eligibility.

**What does not exist:** the canonical `deployments/testnet/manifest.json` is still **not-deployed**. No contract ID, signed Testnet deployment, actual token/privacy verifier, issuer-backed asset, custody or fiat payout has been independently attested. The existing manifest v1 is not yet a full three-contract deployment-attestation format.

**Near-term release plan:** verify artifacts and isolated Testnet RPC through the [preflight](deployments/testnet/OPERATOR-DEPLOYMENT-RUNBOOK.md); obtain an explicitly approved operator-controlled public G-address; deploy **only with separately approved local wallet signing**; independently verify actual on-chain WASM, source and constructor-bound registry addresses; record transactions and update a reviewed three-contract manifest before backend/SDK/frontend are allowed to consume live contract IDs.

**Privacy roadmap:** evaluate Confidential Tokens for value confidentiality and Stellar Private Payments for relationship privacy as separate systems. Define issuer authority, private proof inputs, nullifier/replay protections, fee/TTL, metadata disclosure, account recovery and regulator access. Implement fund-moving contracts **only** when the chosen primitive, legal model, asset issuer and external settlement arrangements have passed security review.

**Verification commands:** `cargo test --workspace --locked`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `python3 scripts/artifacts.py`, `python3 scripts/verify_artifacts.py`, and `python3 scripts/preflight_testnet.py`. A local WASM match is not evidence that anything has been deployed.

**Confidential payments. Without borders.**

[Frontend](https://github.com/stealthbridge-labs/stealthbridge-frontend) · [Backend](https://github.com/stealthbridge-labs/stealthbridge-backend) · [SDK](https://github.com/stealthbridge-labs/stealthbridge-sdk)

> **Status:** research-stage Stellar **testnet-only** protocol. Three **public governance Soroban contracts** (corridor registry, policy registry, read-only governance gate) are compiled and tested as source; no verified on-chain deployments, privacy proofs, or fund-moving settlement primitives exist.

## What is here?
- `contracts/corridor-registry`: authorized and pausable on-chain public corridor governance.
- `contracts/policy-registry`: revisioned public commitments and strict commitment matching.
- `contracts/governance-gate`: fail-closed cross-registry and bounded batch reads in Rust.
- \`docs/PROTOCOL-ARCHITECTURE.md\`: confidentiality vs anonymity, issuer-controlled stablecoins, planned boundaries.
- \`docs/TESTPLAN.md\`: formal verification milestones, no credentials required.
- \`deployments/testnet/manifest.json\`: truthfully empty deployment manifest.
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
what is measured and the local test limitations. The checked-in Testnet manifest
remains `not-deployed`: a local build is not evidence of deployment, settlement,
or verified network compatibility. Confirm the target network's protocol before
any separately authorized deployment.

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

Refer to [the policy security model](docs/POLICY-REGISTRY.md) and [the comprehensive roadmap](ROADMAP.md). No contract ID or live asset integration is claimed until independently verifiable deployment evidence exists.

## Detailed implementation guide

[Contract implementation guide](docs/IMPLEMENTATION-GUIDE.md) documents the evolving Soroban workspace, authorization rules, verification procedures, TTL handling and deployment boundaries.

## Source-level contract interface shared with the SDK

[`integrations/public-soroban-interface.v1.json`](integrations/public-soroban-interface.v1.json) inventories the *actual* CorridorRegistry, PolicyRegistry and GovernanceGate read methods and separates them from administrator-controlled writes. CI checks the public method names against Rust/Soroban source before an SDK can mirror the interface. This is a source ABI reference, **not** evidence of an on-chain contract instance, a privacy circuit, or a live transfer.
