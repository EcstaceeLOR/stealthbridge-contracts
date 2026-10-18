# Three-contract governance attestation — consumer handoff

The source-controlled observation at `deployments/testnet/governance-deployment-evidence.v2.json` documents the 17 October 2026 Testnet deployment. It is **historical evidence**, not a signed, canonical allowlist or a promise of currently valid contract state. The legacy `manifest.json` remains schema v1 and must not be silently overwritten by an observation.

## Required v2 release gates

1. An independent operator verifies the Testnet passphrase, current ledger, contract executable WASM hashes, admin G-address, paused flags and governance-gate registry pointers using fresh Stellar RPC reads; note the ledger sequence and timestamp of verification.
2. Recompute the checked-in source/WASM/ABI provenance; ensure contract IDs are unique and match the registry pointers in the gate constructor.
3. Record the operator's explicit approval with a signed digest of a **new immutable canonical manifest v2** and preserve the original observation separately. Key recovery and revocation must be documented; never commit operator secrets.
4. Consumers must verify manifest schema version, exact Testnet passphrase, signatures/approved trust anchor, hashes and validity interval before exposing trusted reads. Invalidation, upgrades, registry rotation, pausing and TTL expiry must cause reads to fail closed.
5. The SDK and backend may report verified **governance status** only. No privacy proof, asset custody, platform-fee collection, fund-moving settlement or fiat-payout capability is inferred.

## Cross-repository delivery contract

- Contracts: own authoritative ABIs, reproducible hashes, independent Testnet observation and the approved canonical release manifest.
- Backend: verify release pin against current RPC state and expose sanitized read-only status with observation ledger/time and explicit disabled payment flags.
- SDK: strict typed decoding of backend attestation status, never treat a C-address or local JSON as proof on its own.
- Frontend: show the difference among deployed, independently verified, stale, paused and unavailable; keep action buttons disabled.

**Exit condition:** fresh independent chain checks plus integration tests passing across all four repositories. A GitHub green check or Vercel deployment alone does not satisfy this gate.
