# ADR-006 — Issuer-controlled confidential asset viability on Stellar

**Decision:** Research only; **NO GO** for StealthBridge payment execution, public privacy assurances, issued assets, custody or fiat redemption. Adopt a *known counterparties / private amounts* architecture as a candidate only after independent reproducible Testnet proof and issuer authorization. Do not conflate Confidential Tokens with private-address Stellar Private Payments.

**Review snapshot:** 2026-10-13. Primary source: OpenZeppelin [`stellar-contracts` commit `6b2f03402b6b98fad6b4247b5e15dce91a0b205d`](https://github.com/OpenZeppelin/stellar-contracts/tree/6b2f03402b6b98fad6b4247b5e15dce91a0b205d), workspace version **0.7.1** with `soroban-sdk = 28.0.0`. StealthBridge's separately pinned local Soroban registry toolchain is **SDK 27.0.6 / protocol target 27 / Stellar CLI 27.1.0**: different package versions and protocol assumptions. **No claim of binary or API compatibility**. The upstream [`examples/confidential/README.md`](https://github.com/OpenZeppelin/stellar-contracts/blob/6b2f03402b6b98fad6b4247b5e15dce91a0b205d/examples/confidential/README.md) uses CLI **28.1.0** and describes testnet protocol 29. Any cross-contract bridge requires a new toolchain compatibility study and a reviewed proof verifier dependency.

## Primary source evidence

- [OpenZeppelin Confidential Token module](https://github.com/OpenZeppelin/stellar-contracts/blob/6b2f03402b6b98fad6b4247b5e15dce91a0b205d/packages/confidential/README.md) — Grumpkin Pedersen commitments, Noir/UltraHonk proof verification, auditor/view key structure, public counterparties; experimental source, not a stable packaged SDK.
- [Compliance extension specification](https://github.com/OpenZeppelin/stellar-contracts/blob/6b2f03402b6b98fad6b4247b5e15dce91a0b205d/packages/confidential/docs/compliance.md) — optional policy hook, SAC authorization passthrough, per-account freeze, managed clawback and admin controls. **None are automatically enabled** for a generic wrapper.
- [Protocol system and trust assumptions](https://github.com/OpenZeppelin/stellar-contracts/blob/6b2f03402b6b98fad6b4247b5e15dce91a0b205d/packages/confidential/docs/protocol/system-model.md) — underlying asset must be non-rebasing, no fee-on-transfer, revert deterministically, plus honest verifier/SRS and correct auditor key custody.
- [Protocol security analysis](https://github.com/OpenZeppelin/stellar-contracts/blob/6b2f03402b6b98fad6b4247b5e15dce91a0b205d/packages/confidential/docs/protocol/security.md) — the accounting model and transfer proof assumptions; mathematical arguments in upstream docs are **not** our independent cryptographic audit.
- [Stellar privacy guidance](https://developers.stellar.org/docs/build/apps/privacy) and [Stellar smart-contract overview](https://developers.stellar.org/docs/build/smart-contracts/overview) — separate Confidential Tokens and Stellar Private Payments; native Stellar Asset Contract retains issuer authorization and clawback operations where configured.
- [Stellar developer preview](https://stellar.org/blog/developers/developer-preview-confidential-tokens-on-stellar) — Testnet demonstration context, not permission for production use.
- [Nethermind SPP source `0471e6be8161c2013cfc8387bd7b72b70e480cf5`](https://github.com/NethermindEth/stellar-private-payments/tree/0471e6be8161c2013cfc8387bd7b72b70e480cf5) — separate Groth16 privacy pools, alpha and unaudited; do not substitute SPP's counterparty-shielding semantics for this issuer model.

These source observations were reviewed against exact commits; future changes to verifier keys, circuits, API, protocol, compliance hooks or assets require a new ADR review.

## Component and privilege matrix

| Actor | Permitted actions in *candidate* system | Visibility | Privilege compromise / risk |
| --- | --- | --- | --- |
| SEP-41/SAC issuer | Issue/mint and burn/redeem underlying asset, set trustline authorization and clawback **if supported and enabled by actual issuer** | Public issue/redeem edges, trustlines, account identities, amounts at transparent boundaries | Issuer can change base-asset supply/authorization; StealthBridge cannot override its legal duties or controls |
| Confidential wrapper admin | Configure optional freeze/policy, approved policy rotation, maybe clawback | Account addresses, public state/metadata and operational timing; not automatic plaintext amount access | Compromised role can deny service, freeze assets or abuse configuration. Require multisig, timelock, change review |
| Verifier-key registry manager | Approve or rotate circuit verification keys | Published verification-key hashes, governance history | Malicious or accidental key update can undermine soundness. Prefer immutable VKs or timelocked break-glass control |
| Independent auditor | Hold off-chain Grumpkin viewing material; decrypt explicitly designated disclosures where supported | Authorized ciphertext/balance and transfer data; key material must remain private | Auditor-key theft deanonymizes scoped values; the example's secret-key-1 generator must **never** be used in production |
| Policy/allowlist administrator | Manage eligible identities and policy transitions | Known identities, approved/blocked membership, eligibility and policy events | Privilege misuse causes censorship or false eligibility; independent reviews and revocation records required |
| Holder wallet | Authorize own deposit, transfer, withdrawal and selective disclosure; custody proof/wallet secrets locally | Own balances and transaction details; counterparties are **not anonymous** | Stolen wallet/privacy-state material and misdirected signing cause irrevocable loss or disclosure |
| StealthBridge orchestration | Read network/catalog, perform off-chain compliance/reconciliation *only after approval* | Currently only public Testnet ledger metadata and undeployed local registry source | Must not gain custody, auditor keys or unilateral issuer control; no payment routes enabled |
| Licensed fiat partner | Off-chain KYC/AML, liquidity, FX, bank delivery, reconciliation and reversals | Real-world payer/payee and payout details as legally required | Availability, legal, reserve and delivery obligations are not enforced by the Soroban contract |

Separation of duties must forbid a single online key from controlling issuer mint/burn, verifier upgrades, auditor key registration, funds movement and payout confirmations.

## Capability and trust-boundary matrix

| Capability | What upstream source currently describes | What StealthBridge has verified | Go-live owner / blocking evidence |
| --- | --- | --- | --- |
| Mint / burn | **Underlying SEP-41/SAC** issuer operations, not unrestricted mint of the confidential wrapper | **Not verified** | Actual issuer approval; supply-invariant reconciliation; exact transaction evidence |
| Deposit / withdraw | Transparent underlying token enters/leaves the wrapper; value and participants observable at edges | **Not verified** | Proven underlying asset behavior and code hashes; fee/rebase negative tests |
| Amount-confidential transfer | Proof-verified private value within known sender/receiver addresses | **Not verified** | ZK soundness review; transfer vectors; observer logs; independent wallet cross-check |
| Freeze / authorize | Optional wrapper freeze, policy allowlist and SAC passthrough; not universal defaults | **Not verified** | Runtime configuration evidence and unauthorized/revoked-role tests |
| Clawback | Optional, role-gated custody flow with auditor interaction | **Not verified** | Legal authority, proof circuit and non-replay tests; demonstrate recipient accounting/recovery |
| Viewing authority | Auditor Grumpkin ciphertext and key registration, selective disclosure design | **Not verified** | Independent authorized decrypt and failed unauthorized-view test; key loss/rotation drill |
| Admin rotation / upgrades | Sample role gates/config changes and proof-key registry | **Not verified** | Dual-control authority, delay, timelock and rollback drill; exact executable hashes |
| Total supply conservation | Commitments vs wrapper-held underlying amount, conditional on underlying token assumptions | **Not verified** | Independent invariant and hostile asset tests across mint/burn/transfer/withdraw |
| Fiat redemption / reserves | **Outside** Soroban confidentiality primitive; regulated issuer and bank obligations | **No issuer/partner** | Issuer agreement, reserve proof, settlement/provider reconciliation, legal/compliance signoff |
| Mainnet privacy assurance | Developer-preview design only | **Not supported** | Separate external audit and platform/protocol approvals |

**Decision record:** Keep backend `payments_enabled=false`, `confidential_token_verified=false`, `fiat_payouts_enabled=false` and Soroban `status=not-deployed` no matter how many unit/WASM tests pass. Deployment requires independent Testnet evidence, not a green CI badge.

## Disclosure and linkability matrix

| Operation | Sender/recipient | Amount/balance | Public event/metadata | Remaining correlation |
| --- | --- | --- | --- | --- |
| SEP-41/SAC mint or burn | Public issuer and recipient/source | **Public** underlying amount | Asset issuance / account operations | Origin, timing, trustline history |
| Deposit to wrapper | Public holder and wrapper | **Public** amount at entry | Invocation and underlying token transfer | Associates public account, asset and deposit size |
| Confidential transfer | **Both account addresses visible** | Intended ciphertext/commitment; cannot claim zero leakage without testing | Invocation, proof bytes, auditor ciphertext, verification outcome, timing | Counterparty graph, amounts inferred from edge/repeated behavior, wallet metadata |
| Withdraw | Public holder and destination | **Public** transparent exit amount | Contract invocation and SAC transfer | Deposit/withdraw edge correlation |
| Freeze / blocklist | Affected account and admin/control address observable | Usually no plaintext value required | State changes, authorization actions, execution timing | Membership lists and policy changes expose institutions |
| Clawback / auditor disclosure | Governed account identities and privileged role observable | Authorized auditor sees value; clawback exit to issuer transparent | Special proof/authorization and settlement receipt | Confidentiality against auditor/issuer does **not** apply |
| Fiat payout / bank redemption | Off-chain verified sender, receiver and provider | Provider sees real-world amount/FX under applicable regulation | No universal on-chain proof of off-chain delivery | Banking records and partner operations outside chain privacy |

Never assert that encrypted amount alone implies anonymous sender/receiver, unobservable relationship or final fiat payout.

## Threat and mitigation register

| Risk | Severity | Concrete adversarial test or mitigation |
| --- | --- | --- |
| Wrong verification key or malicious registry update | Critical | Rejected forged proof, altered VK fails governance, timelocked recovery; pin keys to reviewed circuits |
| Wrapped token rebases/taxes transfer or unexpectedly fails | Critical | Mock fee-on-transfer/rebase/false success; assert invariant and fail-close; refuse incompatible SEP-41 tokens |
| Freeze/policy bypass and compromised admin | Critical | Unauthorized role and wrong-tenant attempts fail, paused operation never changes balances; multi-signature change controls |
| Auditor key compromised or globally shared | High | Unauthorized decrypt fails, rotation prevents future viewing, archived disclosures access-controlled; separate escrow policy |
| Amount inference from public deposit/withdraw boundaries | High | Observer compares RPC/events to ground truth; prohibit categorical confidentiality claims for edge operations |
| Supply/reserve mismatch | Critical | On-chain wrapper accounting vs underlying balance independently reconciled; fiat reserve/legal attestation off-chain |
| Replay/double spend/stale proof | Critical | Wrong nonce/state root, replayed proof, expired authorization and duplicated submission fail without mutation |
| Counterparty graph/policy privacy | High | Identify all public addresses and allowlist changes; strictly limit personal identifiers/events |
| Storage TTL and restoration | High | Archive/revive affected keys, verify ledger costs and account state after resumption |
| Protocol/toolchain drift | High | Reproduce exact upstream commit/tool versions; don't compose SDK 27.0.6 with upstream SDK 28 without verified compatibility |
| Reconciliation/provider callback false completion | Critical | Idempotent signed scoped callback, cross-tenant rejection, proof of actual bank payout; never equate RPC success with fiat delivery |

## Executable future acceptance checklist (none complete)

- [ ] Independently reproduce upstream v0.7.1 Confidential Token + UltraHonk verifier at pinned SHA and protocol/SDK, record artifact hashes and proving-key verification.
- [ ] Authenticate valid issuer and holder identities; prove only authorized issuer can mint/burn the *underlying* asset. Reject wrong issuer and revoked/expired trustline.
- [ ] Register two distinct real Testnet wallet accounts and distinct auditor key; test wrong-wallet, signature replay and unrelated account authorization.
- [ ] Deposit exact Testnet amount; independently inspect `getTransaction`, events and wallet data, record public amount and receiver identities.
- [ ] Transfer privately; check recipient balance/proof, verifier soundness and conflicting/duplicate proof rejection, and independently inspect public address/amount leakage.
- [ ] Withdraw and compare transparent exit to commitment conservation; reject fee-on-transfer, rebasing and incorrect/missing underlying asset.
- [ ] Exercise role-revocation, freeze, delayed policy rotation, malicious key-registry update, threshold authorization, account recovery, lost/auditor key.
- [ ] Simulate auditor viewing and selective disclosure, show unauthorized viewers cannot decrypt, record who still sees public addresses and metadata.
- [ ] Verify on-chain contract IDs/WASM/protocol, observed transaction hashes and Testnet ledger freshness against two independent observers.
- [ ] Independent security review and licensed issuer/partner readiness; investigate reserve reporting, redemption rights, sanctions, KYC/AML and third-party payout evidence.
- [ ] Confirm compatibility or explicitly reject shared Soroban registry protocol/tooling; repeat all CI gates after any source/CircuitType/verifier change.
- [ ] Review the [Testnet evidence chain](TESTNET-DEPLOYMENT-EVIDENCE.md); only a new independently reviewed release gate may change capability flags.

## Resolution and revisiting this ADR

This ADR completes the **research/governance threat model**, not the cryptographic reproduction tracked in #2 or the executable privacy gates in #11. It explicitly **rejects** assuming a live USDC-private asset, available issuer, production-grade proving system, escrow, reserves, privacy guarantees or integrated bank payout.
