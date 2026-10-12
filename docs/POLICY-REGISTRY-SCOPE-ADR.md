# ADR: Public Policy Registry and authorization boundaries

Status: implemented local Soroban Testnet **source-only prototype**, with no claimed Testnet deployment, audit, regulated issuer role, or confidential payment proof.

## Public data model

The Rust `PolicyRegistry` stores a public operator-controlled rule identifier (`String`, 1–128 bytes), monotonic non-zero `revision` (`u32`), a Boolean `enabled`, and an opaque 32-byte **public** commitment. These values are intentionally not people, KYC verdicts, sanctions checks, payout account IDs, transaction amounts, decrypted notes, witness material, or private keys. `is_effective_commitment(id, revision, digest)` checks exact revision/commitment and the emergency pause flag. The standalone `GovernanceGate` combines source-only corridor and policy reads; it does not attest asset issuance or authorize transactions.

## Authorized governance

A constructor requires admin authorization. Only the current admin may nominate/cancel a successor or change the pause and policy records; a nominated successor must independently authorize acceptance. Stale administrators and unscoped signatures cannot continue to mutate a registry. The pause flag defaults to **fail closed** if state is missing. While paused, enabled policies cannot be staged, but monotonically higher revision revocations can still be stored for incident response. Re-enablement after revocation requires another reviewed policy revision.

Both instance and persistent Soroban storage use `extend_ttl(TTL_THRESHOLD, TTL_EXTEND)`, with `TTL_THRESHOLD=17,280` and `TTL_EXTEND=518,400` ledger sequences. Operators must budget TTL rent and restoration operations separately. A local simulator's auto-restoration behavior is not an executed restoration transaction.

## Threat model and limitations

| Attacker or failed assumption | Public control | What is **not** enforced |
| --- | --- | --- |
| Unauthenticated outsider | `require_auth` for all governance changes | Off-chain legal membership verification |
| Stale former admin | Two-phase nomination and successor-accepted transfer | Custody/recovery of the new admin's actual private keys |
| Bad public rule revision | Reject zero or non-monotonic revision | KYC, sanctions, off-chain approvals or policy meaning |
| Missing/corrupted pause state | Default to paused | Cross-chain finality and confidential balance validity |
| Expired contract storage | Explicit TTL and restore runbooks | Automatic chain restoration |
| Compromised administrator | Emergency pause/revocation, documented role separation | Independent institutional multisignature approvals without new audited code |

## Acceptance and regression suite

`cargo fmt --all -- --check`, `cargo test --workspace --locked`, `cargo clippy --workspace --all-targets --locked -- -D warnings` and both isolated WASM builds are CI gates. Policy tests cover revision rollback, unsupported identifiers, non-admin mutation, nomination/cancellation, stale signer, pause and recovery, and deterministic property matrix over identifier sizes 1/32/64/128 and revisions 1–12. These are source tests; they do not claim real money, confidential transfer, deployed contracts or external audit.

## Next phases

A separately reviewed, independent protocol and compliance integration may add scoped organization identities and operator governance, but must never write protected personal/witness data into Soroban storage or event topics. This ADR does not approve a private payment rail, mint/burn authority, fiat payout, or issuer partnership.
