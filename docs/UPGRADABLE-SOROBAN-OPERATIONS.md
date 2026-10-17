# StealthBridge Testnet deployment and upgrade operations

**Status: no contract deployment claimed. This is a reviewed operator procedure, not a request for wallet secrets or authorization to spend.** All three current contracts are public Soroban governance scaffolding, not confidential payment or token settlement contracts.

## Why this upgrade design

- `corridor-registry` and `policy-registry` are pausable and have an administrator-controlled *propose → inspect → wait → execute or cancel* WASM upgrade lifecycle. A proposal stores only a public 32-byte WASM hash and ledger window. Both proposal and execution require Soroban `require_auth` of the active administrator. The proposal is valid no earlier than **5** and no later than **20,000** ledgers after creation, and execution expires 10,000 ledgers after its ready ledger.
- An upgrade **must remain paused**, use the exact previously staged WASM hash, and invoke `env.deployer().update_current_contract_wasm`. Updating an executable preserves the contract address and stored entries but **does not** rerun its constructor or guarantee schema compatibility.
- `governance-gate` is deliberately **immutable** and only reads public states of the two registries bound at construction. Gate changes require a separately reviewed new gate address and coordinated backend/SDK/manifest migration, rather than an invisible mutable trust anchor.
- These controls provide **delay, not multisig**. Before accepting any mainnet use, hold administration under a reviewed multisig/organization account policy, rotate/recover keys securely, audit storage migrations and contract semantics, rehearse rollback/recovery paths, and obtain independent sign-off. A malicious replacement WASM is still dangerous after a correctly signed upgrade.

Stellar documents that `update_current_contract_wasm` works with a previously uploaded executable and retains the contract ID; its exact bytecode hash must be the one independently reviewed: https://developers.stellar.org/docs/build/guides/conventions/upgrading-contracts .

## Prepare entirely offline — no signing, no secrets

Use the repository's pinned Rust toolchain, installed `wasm32v1-none` target, pinned Stellar CLI **27.1.0**, and the project Soroban SDK **27.0.6**. Reconfirm the target network's current protocol before you submit anything.

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --locked --release --target wasm32v1-none
python3 scripts/artifacts.py
python3 scripts/verify_artifacts.py
python3 scripts/preflight_testnet.py --operator YOUR_PUBLIC_G_ADDRESS
python3 scripts/plan_testnet_release.py --identity YOUR_LOCAL_KEY_ALIAS --admin YOUR_PUBLIC_G_ADDRESS
```

The planner validates clean, two-build reproducible artifacts, all three SHA-256 fingerprints, pinned protocol/toolchain metadata, and your **public** administrator address. It prints shell commands and **never runs them**.

## First deploy — only from your own machine, signed locally

1. Create or import a **dedicated Testnet operator identity** into your Stellar CLI, with secret material retained locally in an OS keychain/hardware wallet. For a new disposable Testnet key, the official tutorial uses `stellar keys generate operator --network testnet --fund`; replace `operator` with the alias you've chosen. Never put a seed phrase in chat, GitHub, a web form, a command-line argument, or CI variables for this project. A public `G...` address is safe to review.
2. Verify `stellar keys address YOUR_LOCAL_KEY_ALIAS` returns the public `G...` address that should administer the registry. Run the offline steps above.
3. Independently inspect ABI/constructor arguments in the local WASM and **approve actual signatures** before broadcasting. Deploy `corridor-registry` and `policy-registry` first, recording their independently observed `C...` contract IDs and signed transaction hashes; then deploy `governance-gate` with those exact registry IDs as its constructor arguments.
4. For each contract, compare `stellar contract info hash --contract-id C... --network testnet` to `artifacts/provenance.json` and call its read-only admin/registry getters. Do not mark the manifest as deployed based on CLI output alone; independently replay each network read from the selected RPC with network-passphrase checking.
5. Prepare the reviewed three-contract attestation schema and transaction evidence. The **current** `deployments/testnet/manifest.json` intentionally remains `not-deployed` and must not be edited by an automatic deploy script. The backend currently rejects fabricated on-chain deployment claims.

Official Stellar CLI usage: https://developers.stellar.org/docs/build/smart-contracts/getting-started/deploy-to-testnet and https://developers.stellar.org/docs/tools/cli/stellar-cli .

## Subsequent paused WASM upgrade — two registries only

```sh
python3 scripts/plan_testnet_release.py \
  --identity YOUR_LOCAL_KEY_ALIAS \
  --upgrade-contract policy-registry \
  --contract-id VERIFIED_EXISTING_POLICY_C_ADDRESS \
  --new-wasm artifacts/stealthbridge_policy_registry.wasm \
  --delay-ledgers 120
```

For the corridor registry, select `corridor-registry` and its matching existing `C...` address/WASM. This command prints a **plan**. It refuses malformed contract addresses, unknown contract types, impossible delay intervals, and WASM that differs from the reproducible build. It makes **no** on-chain status claim and **never submits or signs**.

After manual review: verify the current deployed hash and admin, upload the *new* reviewed WASM with your locally stored CLI signer, explicitly pause the registry, propose the newly uploaded hash, independently inspect `pending_upgrade`, wait for `ready_at_ledger`, check the ledger is at or before `expires_at_ledger`, and only then execute with a **separate authorized signature**. Re-query the same contract address, resulting deployed hash and persistent state (including exact policy commitments), and keep it paused through post-upgrade verification. Cancel an unwanted proposal with an authenticated `cancel_upgrade`. Unpausing is always a separate operator decision.

**Important:** the planner cannot prove the existing contract identity, exact on-chain state, ABI compatibility, correct RPC passphrase, current paused flag, upload receipt, administrator control, migration compatibility, or completed upgrade. Those are mandatory manual/independent acceptance gates, not implied by successful local tests.

## Future production hardening

Mainnet release requires versioned storage migrations and compatibility/rollback rehearsals; a release multisig with role segregation; independent security review; contract events for proposal/execution attribution; TTL/rent and failed-upload operational drills; bounded on-chain costs; upgrade-delay monitoring; shadow reads comparing SDK/backend behavior; alerting; post-release gradual activation; and a staged proof verifier and legally supported payment rail.

**There is no wallet/seed/private-key request, fund transfer, private witness, or automatic deployment in this runbook.**
