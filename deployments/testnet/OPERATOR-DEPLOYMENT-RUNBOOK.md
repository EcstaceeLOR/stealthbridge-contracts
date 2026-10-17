# Testnet governance-contract deployment ceremony (operator approval required)

> **Release update, 17 October 2026:** This original prospective ceremony
> has been completed for the three public governance contracts with
> `operator_local` as both Testnet signer and administrator. See the
> [on-chain verification and transaction receipts](VERIFIED-GOVERNANCE-RELEASE-2026-10-17.md).
> The older `not-deployed` statements below are a historical runbook baseline,
> not a claim about the live Testnet ledger. Further upgrades or Mainnet
> activity require separate reviews and authorization.

**Status: not deployed.** This is a reviewed sequence for a future, explicit
operator-approved Testnet deployment. The scripts here do **not** perform
uploads, contract deployment, key generation, account funding, signing, or
financial operations. Current manifests and public APIs must remain
`not-deployed` until independent on-chain attestation succeeds.

## Before using a wallet

Use a dedicated, funded **Stellar Testnet** administration account, with
separate organizational custody approvals. The public account must be a
checksum-valid classic `G...` address. **Never supply or commit** a Stellar
secret (`S...`), seed phrase, key export, Freighter recovery phrase or
hardware-wallet credential to the repository, any API endpoint, CI, Vercel
environment, or chat. Do not confuse website **Connect Freighter** with
authorization to deploy contracts: the frontend is intentionally read-only.

If an approved external signer will be used, configure its identity in the
operator's own Stellar CLI or use [Stellar Lab](https://lab.stellar.org/)
with explicit transaction approval. Signing belongs on the operator's device.

## Step 1: reproduce and verify the artifacts

Check out the *exact* reviewed source commit whose CI artifact was approved.
Use the pinned Rust 1.91.0, Soroban SDK 27.0.6 and Stellar CLI 27.1.0
toolchain recorded in the repository CI. Do not silently upgrade the protocol
toolchain immediately before an irreversible deployment.

```sh
python3 scripts/artifacts.py
python3 scripts/verify_artifacts.py
python3 scripts/validate_manifest.py
python3 scripts/preflight_testnet.py
```

The artifact verifier expects three WASM files and their three extracted
interfaces, complete SHA-256 checksums and clean-tree provenance. A valid local
hash is **not** proof of an on-chain deployment. Preserve the CI artifact
archive before funding or signing anything.

## Step 2: check the actual Testnet (read-only)

Set `STELLAR_RPC_URL` securely as an environment variable. Use a newly
rotated provider credential; avoid placing secret-bearing RPC URLs in shell
history or command arguments. For an operator whose public address is stored
in `TESTNET_ADMIN_PUBLIC`, verify:

```sh
python3 scripts/preflight_testnet.py \
  --operator "$TESTNET_ADMIN_PUBLIC" \
  --rpc-url-env STELLAR_RPC_URL
```

This checks RPC `getNetwork`, `getLatestLedger`, the exact Stellar
Testnet network passphrase, minimum contract protocol, ledger hash and
maximum age. It performs **no wallet request** and does not establish
account funding or contract authenticity.

## Step 3: explicit signing/deployment (not automated here)

Only after written operator approval and security review, deploy in this
dependency order with a locally controlled signer and **review each
transaction envelope before approval**:

1. `corridor-registry` — constructor `admin` set to the reviewed
   Testnet administrator G-address; record the returned contract C-address
   and transaction hash from the actual network
2. `policy-registry` — constructor `admin` set to the reviewed
   administrator; record real chain evidence
3. `governance-gate` — constructor `admin`, `corridor_registry`,
   `policy_registry`; both references must exactly match the *already
   attested* deployed registry instances

The official Stellar CLI supports `stellar contract deploy --wasm ...`,
`--source-account <locally-configured-signer>`, `--network testnet`,
and `-- <constructor-args>`. Check the exact pinned CLI
`stellar contract deploy --help` before executing. A connected webpage
Freighter account is **not** passed as an authorized source.

**Deploying an immutable-address gate with the wrong dependency address
requires a new gate deployment.** Do not use test-only C-addresses in
the constructor.

## Step 4: independently attest each deployment

Use a separate reviewer, RPC source and/or Stellar Lab contract explorer to
verify the Testnet network passphrase, successful deployment transactions,
contract StrKey checksum, actual WASM bytes/hash, extracted interfaces,
constructor-bound dependency addresses, administrator state, and TTL. Compare
the three WASM and ABI hashes with `artifacts/provenance.json`.

For a signed, already-deployed Testnet installation, create a **local** JSON
file containing the actual three contract addresses under keys
`corridor-registry`, `policy-registry`, and `governance-gate`, then run:

```sh
python3 scripts/verify_onchain_wasm.py --contracts-file YOUR_LOCAL_CONTRACT_IDS.json
```

This command uses read-only `stellar contract fetch --network testnet`
invocations and checks byte-for-byte WASM equality against the pinned
artifacts. It never requests a signer. Confirm your local Stellar CLI
`testnet` network configuration independently before trusting the fetch.
Bytecode equality alone **does not verify** successful deploy transactions,
admin state, or governance-gate constructor bindings; those require separate
independent on-chain reads and reviewed evidence.
The current older `deployments/testnet/manifest.json` validator supports
local consistency checks and does not certify three on-chain deployments.
**Do not** mark it `deployed` until an updated, independently reviewed
three-contract attestation schema is present and verified in contracts,
backend, SDK, and frontend. A successful contract deploy must not
automatically enable corridor flags or payment capabilities.

## Step 5: gated read-only rollout

After independent on-chain verification, update the canonical manifest and
its pin in backend/SDK, then add RPC-backed contract inspection to backend
and frontend. Verify real `get_admin`, `is_paused`,
`is_enabled`, `get_rule`, and `public_flags_allow` results.

Keep `payment_execution_enabled=false` and fiat payout disabled until
separate audit, privacy proof, issuer/asset, signing authorization,
organization permissions, quote and settlement reviews are complete.

References: [Stellar CLI deployment](https://developers.stellar.org/docs/tools/cli/cookbook/upload-deploy)
and [Stellar Lab contract explorer](https://developers.stellar.org/docs/tools/lab/smart-contracts/contract-explorer).
