# Issues #30 and #32 — fee vault authorization and recovery review

**ISOLATED RESEARCH CONTRACT ONLY. NOT DEPLOYED, AUDITED OR APPROVED FOR
CUSTOMER FEES. DO NOT ENABLE COLLECTION OR AUTOMATED WITHDRAWAL FLOWS.**
Three governance contracts already deployed on Testnet are separate from
this fee-vault source. Fee transfer events are PUBLIC and not confidential.

## On-chain roles and accounting

- Immutable SEP-41 token contract address from constructor; admin explicitly
  signs constructor. Vault is initially paused.
- Administrator controls deposit pause, **independent claim pause**, succession,
  beneficiary nomination and hash-pinned delayed WASM proposals. Beneficiary alone signs withdrawals.
  Initially beneficiary = admin; after handover their roles are independent.
- `deposit_fee(payer, amount)` requires payer authorization and an explicit
  positive amount. Deposits occur only if the local prototype was unpaused.
  A separate authorized platform payment rail/fee processor **does not
  exist** in this repo. Unpausing this research vault is NOT a release gate.
- `claimable_fees()` tracks only successful explicit deposits, not unsolicited
  direct token transfers. Token balance can exceed accounted fees.
  Overflows, underflows, overclaims, and nested token callbacks fail closed.
  Claims debit accounting before calling the token; Soroban atomic rollback
  protects the state when that call fails.
- `claim_fees(amount)` requires BENEFICIARY auth, sends only to that stored
  beneficiary, and accepts no caller-provided destination. Claims start
  **paused independently** from deposits. Administrator must explicitly call
  `set_claims_paused(false)` after review. Deposits may be paused while an
  authorized beneficiary performs an approved claim; a suspected compromised
  beneficiary must be blocked by `set_claims_paused(true)` before rotation.
  Do not activate or fund this unaudited prototype.
- Administrator and beneficiary rotations are independent two-phase
  `propose_*` then `accept_*` operations, with cancellation. An old
  beneficiary cannot authorize claims after a completed beneficiary change.
  Admin compromise before recovery could nominate an attacker; a single
  signature is NOT sufficient production governance.
- Contract emits fee deposit/claim, beneficiary rotation and upgrade-staging
  events. Token transaction history is public and must be reconciled against
  claimable accounting and any direct unsolicited balance.
- Upgrade requires a **paused vault**, admin auth, a nonzero reviewed WASM
  hash, 5–20,000-LEDGER delay, exact hash match and 10,000-ledger expiry.
  A wrong hash, premature, expired, missing or unpaused upgrade is rejected.
  Replacing WASM does NOT rerun constructor and may break storage layout.

## Disaster recovery and change-control drill

1. Establish a reviewed recoverable **multisig/contract-account signer**
   before any custody. Protect multiple distinct signer devices; back up
   encrypted seeds offline and rehearse offline restoration using a
   dedicated disposable Testnet identity. Never store keys in GitHub, Vercel,
   CI settings, logs, commands, or messages.
2. Verify current admin and beneficiary using public getters on the right
   Testnet contract ID; independently confirm hashes/network identity.
   A CLI alias like `operator_local` is not a recovery mechanism.
3. On lost/compromised beneficiary device, current admin pauses **both**
   deposits and claims, proposes a new verified beneficiary, and the new
   beneficiary accepts. Independently re-check getters and events before
   authorizing any future claim. Unpausing deposits does NOT unpause claims.
   A compromised admin can unpause claims; a reviewed signer quorum remains
   mandatory for financial use.
4. On lost admin device, use the admin's **existing externally configured
   multisig recovery policy**; without a surviving authorized signer, the
   contract CANNOT create an admin out of thin air. Admin handover requires
   the old active admin signature followed by the successor's authorization.
   Maintain independent hardware/backup quorum **before** losing the old key.
5. Before any upgrade: pause, compare reviewed bytecode, stage exact hash,
   independently record readiness ledger and expiring window; require
   out-of-band human approval and immutable security review references.
   Cancel if bytes, signer, token behavior or storage compatibility differ.
   Review successor code migration thoroughly. Rollback is a *second*
   separately reviewed staged upgrade, not an automatic undo.
6. Tests: `cargo test --manifest-path contracts/fee-vault/Cargo.toml`,
   `cargo fmt --manifest-path contracts/fee-vault/Cargo.toml --all -- --check`,
   `cargo clippy --manifest-path contracts/fee-vault/Cargo.toml --all-targets -- -D warnings`
   and WASM build. CI must report green before claiming source is reviewed.

## Not delivered by this branch

No externally audited custody logic, real multisig on-chain quorum, KYC/
issuer authorization, signed per-payment quote and nonce handling, fiat
settlement, fund custody approval, recovery rehearsal evidence, public
Testnet fee-vault deployment or secure WASM rollback execution has been
demonstrated. Claims/collection must remain disabled in production
customer flows. Issues #30 and #32 require these outstanding approvals
before final closure.
