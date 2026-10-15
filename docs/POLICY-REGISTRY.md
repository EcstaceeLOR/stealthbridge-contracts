# Protocol policy registry — security and scope

## Why a separate contract?

Administrative policy *configuration* should not be confused with financial value movement, cryptographic validity, issuer controls or private beneficiary verification. The minimal Soroban PolicyRegistry stores only opaque public identifiers, public commitments and flags, and version-enforces updates.

## Security invariants

1. Every mutation requires an authorized administrator.
2. Revision 0 is invalid and every update must strictly increase revision, including re-enablement after a pause.
3. A paused registry returns false from `is_effective`.
4. Absent entries fail closed.
5. Public commitments may still correlate on-chain activity, so do not submit personal data, account links, KYC records, private witnesses or off-chain identity documents.

## Limitations

The contract does not validate the contents of a commitment or satisfy compliance requirements. There is no issuer-specific verifier, role delegation, proof check, rollback governor, Timelock, access recovery or audited upgrade model. These are future carefully reviewed tasks; no actual fund-moving pathways should reference the registry yet.

## Source

[contracts/policy-registry/src/lib.rs](../contracts/policy-registry/src/lib.rs)

## Recent governance hardening

The PolicyRegistry now supports a **two-phase administrator transfer**, with `propose_admin(successor)`, `pending_admin()`, `cancel_admin_proposal()` and `accept_admin()`. The current administrator authorizes nomination/cancellation; the nominated account must authorize acceptance. Pending nominations confer no administration until accepted. Both registry contracts reject empty or overlong (more than 128-byte) opaque rule/corridor identifiers on writes and fail closed on invalid read identifiers. The policy's public commitment is **not** a verified user eligibility or ZK-proof check.

## Authorization-bound transition test

The policy-registry unit suite now uses Soroban SDK `MockAuth` with explicit contract/function/argument scopes to confirm a wrong signer cannot propose control, the current administrator cannot accept a successor's nomination, and the former administrator loses mutating access after a valid successor authorization. This is a unit-test impersonation model, **not** proof of independently signed mainnet transactions or a deployed/verified policy registry.

## Reject zero public commitments at every registry boundary

Soroban's public-governance approvals now reject an all-zero `BytesN<32>` when **enabling** a policy or approving a corridor's operator-specified config. In a real review, a zero hash cannot identify a reviewed policy document, contract or configuration revision. The public reads `is_effective_commitment`, `is_enabled_with_digest` and `public_flags_allow_commitment` also return `false` if the caller supplies a zero expected commitment, even when unrelated enablement flags are true.

Emergency revocation remains possible: disabling an enabled policy with a zero public commitment is permitted while paused, so a compromised approval cannot hold revocation hostage by requiring the operator to publish a new review hash. Disabling a corridor clears a previously approved digest. Typed Soroban errors `InvalidCommitment` and `InvalidConfigDigest` make administrative misuse explicit without changing the public method signatures.

**A nonzero digest is not cryptographic proof of a licensed issuer, privately settled payment, independent audit or policy correctness.** It is only a public equality-binding identifier controlled by the authorized administrator. Current Testnet deployment manifest remains `not-deployed`; no on-chain contract changes are claimed until independently deployed and attested.
