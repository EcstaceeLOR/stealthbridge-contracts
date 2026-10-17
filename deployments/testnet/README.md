# deployments/testnet

## Current Testnet reality (17 October 2026)

**Three public governance Soroban contracts have now been deployed** and their
WASM/admin/dependency state independently fetched on Stellar Testnet. See the
[verified deployment report](VERIFIED-GOVERNANCE-RELEASE-2026-10-17.md) and
[machine-readable observation](governance-deployment-evidence.v2.json).
The old **v1 canonical manifest below has not yet been migrated** to describe
all three contracts. Its `not-deployed` marker is *stale for the observed chain*
but remains a fail-closed integration state for older backend/SDK/frontend
consumers. No live payments or platform-fee collection are enabled.

Validate the newer, observed three-contract record against pinned WASM/ABI:

```sh
# Obtain the pinned 130c577e CI archive rather than rebuilding a later HEAD.
python3 scripts/verify_artifacts.py --artifacts PATH_TO_PINNED_ARTIFACTS
python3 scripts/validate_governance_observation.py --artifacts PATH_TO_PINNED_ARTIFACTS
```

This validator proves static source/hash/schema compatibility, **not** live
transaction inclusion, code freshness or a security audit. Review the on-chain
receipts and readbacks in the linked report.

**17 October 2026 review:** [verified three-contract CI artifacts and outstanding operator steps](TESTNET-RELEASE-STATUS-2026-10-17.md). All three WASM/ABI files match the pinned release, but the administrator signer and live RPC preflight are not verified. No Testnet deployment transaction has been submitted; this page and the canonical manifest deliberately remain `not-deployed`.

The manifest is intentionally `not-deployed`. Validate without network access:

```sh
python3 scripts/validate_manifest.py
```

The schema is enforced by `scripts/validate_manifest.py`. Version 1 requires
`schemaVersion` (integer 1), `network` (`testnet`), `status`, boolean `verified`,
`contractAddresses` object, empty `assetIssuers` object, `txHashes` array, and a
string `notes`. Unknown fields are rejected. For `not-deployed`, addresses and
transaction hashes must be empty and `verified` must be false. No network
passphrase, address or source hash is fabricated for an empty record.

A `deployed` record additionally requires:

- `networkPassphrase`: exactly `Test SDF Network ; September 2015`.
- `contractAddresses`: exactly a `corridor-registry` entry with a valid contract
  StrKey (C prefix, base32 length/version and CRC16-XModem checksum).
- Nonempty, distinct `txHashes`: lowercase 64-character hex transaction hashes.
- `sourceCommit`: a 40-character lowercase Git commit hash.
- `sourceSha256`, `wasmSha256`, `abiSha256`: lowercase SHA-256 hashes matching the
  local clean-build `artifacts/provenance.json`. The validator also hashes the
  WASM and ABI bytes and compares them with the record.

The current **deployment manifest schema** verifies only the corridor registry
for the older schema-v1 deployed fixture. However, the build pipeline now
produces reproducible WASM/ABI hashes for **corridor-registry, policy-registry,
and governance-gate**. Do not mark either newer contract as deployed until the
manifest schema is explicitly extended and independently checks all deployed
addresses, on-chain bytecode and constructor-bound dependencies. Synthetic IDs
exist only inside isolated validator tests and must never be copied into a
deployment manifest.

After a separately authorized Testnet deployment, a maintainer should:

1. Preserve the clean source commit and CI artifact set. Verify `SHA256SUMS` and
   independently rebuild that commit with the pinned toolchain.
2. Obtain the actual successful deployment transaction hash from the submitted
   transaction. Query the correct Testnet RPC, inspect the result/ledger and
   network identity, and confirm the created contract ID.
3. Fetch the deployed WASM and compare its SHA-256 and extracted interface with
   the archived artifacts. Independently inspect the initialized admin and
   governance state. Record evidence links and the verification procedure in
   `notes`; do not include signer secrets.
4. Insert these actual IDs, hashes and passphrase into a reviewed manifest
   change. Set `verified` true only after the independent chain checks succeed.
5. Run the validator with `--artifacts PATH` pointing at that exact artifact set.

The validator proves schema and local hash consistency only. A valid-looking
address or transaction hash does not establish chain existence or successful
deployment, and the script never sets `verified` or submits any transaction.
Keep historical evidence with the versioned release when replacing a record.
