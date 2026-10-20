# Wallet, smart-account, confidential payment and ZK integration

## Product outcome

StealthBridge is building two user-facing journeys:
- **Send**: recipient-aware cross-border payments with sender/recipient relationship privacy where permitted and independently verified.
- **Business**: authorized treasury settlement with amount confidentiality, maker-checker approval, audit controls and explicit provider reconciliation.

A public registry deployment is NOT a customer wallet or a private payment system. Existing `CorridorRegistry`, `PolicyRegistry` and `GovernanceGate` are live Testnet **governance infrastructure**. The current fee vault and settlement adapter have their own unfulfilled release gates. No current code path may sign or route real-value payments.

## Wallet integration: implemented interface, future programmable signer

| Layer | Integration | Operational state |
| --- | --- | --- |
| Classic accounts | Freighter `@stellar/freighter-api` requestAccess, isAllowed, getAddress, getNetwork | Real public-wallet connection on the StealthBridge web client; signing disabled |
| Public account lookup | Consent-based Stellar Testnet Horizon `GET /accounts/{G}` | Real public data, no custody or financial ability |
| Smart account addresses | StrKey-validated `C…` identity classification | Address format only, not proof of account contract, ownership or signers |
| Smart account authorization | **OpenZeppelin smart-account model via Stellar Smart Account Kit**, passkeys + configurable policy/context rules | Needs reviewed WASM hash, verifier deployment, audited authorization implementation, verified relayer and recovery plan |
| Privacy rail | Confidential Tokens (private amounts, public counterparties) and Stellar Private Payments (private counterparty links within pools) | External preview/alpha; no verified StealthBridge payment integration |
| Payment execution | Contract checks, ZK verification, issuer/regulatory/payout approval and safe release | Disabled |

### User-wallet integration and restricted account research

The primary /send and /business pages now use an actual Freighter-connected
Testnet workspace rather than presenting historical deployment receipts.
The frontend checks network freshness and never asks for private keys.

The backend implements consent-based GET /v1/wallet/contracts/{address} using
a checksum-valid C-address, RPC getLedgerEntries, entry TTL and current
executable WASM hash. The SDK and frontend accept strictly checked responses
only. This verifies an active public contract instance, not custom-account
__check_auth behavior or ownership. A missing C-address does not authorize
creating a wallet, and a found one does not authorize payments.

An isolated source-only experiment lives at
experiments/restricted-account. It explores Ed25519 signature validation
with one pinned contract and method authorization context, and has independent
Soroban native/WASM CI. This code is deliberately outside the three deployed
governance contracts and deployment manifests. It is **not** a customer wallet,
does not implement passkeys or recovery, and is not approved for real value.
Before enabling, require a pinned reviewed Smart Account Kit / OpenZeppelin
account, tested WebAuthn verifier, approved factory and relayer, recovery,
multi-party policy tests, and independent review.

### Smart accounts: architecture decision

Use **Stellar Smart Account Kit + OpenZeppelin Smart Account** as the intended passkey and team-policy model; do not conflate this with the StealthBridge GovernanceGate contract. Smart accounts are user-owned **contract accounts** implementing Soroban `__check_auth`, not a database alias for a G-address. Business deployments should support scoped signers, multiple approvers, recovery and revocation. Consumer deployments should offer passkey device setup and safe account recovery. Neither the application operator nor fee-vault deployer should be a universal user-account signer.

Before a passkey creation button may be enabled, require independently pinned and tested `accountWasmHash`, `webauthnVerifierAddress`, relayer permissions, network passphrase, upgrade policy, recovery policy and verified deployed account-interface code. Sponsor/relayer credentials must remain server-side. Do not create a passkey without a verified account deployment path: a stranded credential is not a usable wallet.

References:
- https://developers.stellar.org/docs/build/guides/contract-accounts/smart-wallets
- https://github.com/stellar/smart-account-kit
- https://docs.openzeppelin.com/stellar-contracts/accounts/smart-account

## Privacy requirements must be differentiated

**Amount confidentiality (Business, Confidential Tokens):** An amount-confidential token can conceal balances and transfer amounts while **exposing sender and recipient addresses**. This is *not* transaction relationship anonymity. The Stellar/OpenZeppelin Confidential Token suite remains a **developer preview**, with audits and integration requirements outstanding. Pin a specific token implementation, issuer, UltraHonk verifier, verifying-key commitments, proof encoding, ledger evidence, upgrade authority, controlled disclosure and test vectors before accepting it as a rail.

**Relationship privacy (Send, Private Payments):** A privacy pool can hide the on-chain link between a deposit and internal withdrawal, but deposits/withdrawals at pool edges may remain visible. Require an inspected pool contract and SDK version, audited nullifier/commitment verification, replay safety, member inclusion, withdrawal authorization, viewer/recovery controls, retention and compliance boundaries. Its *alpha* SDK must not be assumed to have the security properties of an audited production gateway.

Low-level BN254/Poseidon Soroban host functions and Groth16/UltraHonk verifier implementations are cryptographic primitives; they are **not a proof of real application confidentiality** by themselves. No fake ZK verification, placeholder proofs, opaque booleans, or hardcoded external transaction IDs may substitute for on-chain evidence.

References:
- https://developers.stellar.org/docs/build/apps/privacy
- https://stellar.org/blog/developers/developer-preview-confidential-tokens-on-stellar
- https://stellar.org/blog/developers/developer-preview-stellar-private-payments

## Connected end-to-end release path

1. **Onboarding:** Real Freighter public address / connected Testnet session or independently deployed passkey smart account. Verify the selected address/network after extension changes; never handle private seeds.
2. **Planning:** Confirm real corridor record, supported asset, issuer, provider, required compliance disclosure, fees and time-to-completion. Do not fabricate quotes or FX rates.
3. **Authorization:** Pin source ABI, verifier hash, signer policies, approved smart-account auth context and nonce rules; reject wrong network, stale ledger, revoked auth, wrong recipient and self-payments.
4. **Private proof generation:** Client/witness service bound to a specific audited rail version and actual commitment domain. Private witness and amounts must not be persisted in plain logs.
5. **Verification:** Verify the proof on the exact pinned on-chain verifier, with actual verified key, replay checks and pre-execution business policy. A status flag or historic deployment receipt is not enough.
6. **Settlement:** Provider/issuer custody and authorization, actual transaction submission with idempotency, TTL and safe retries; on-chain observation does **not** itself prove fiat payout.
7. **Reconciliation:** One running leased public ledger observer, SQLx-ready Neon and provider-specific payout receipts, with explicit fee vault accounting. Never permit deployer fee claims from unverified or self-reported journal entries.
8. **Release:** Mainnet only after threat-model update, third-party review, full integration tests, legal/compliance readiness and independent on-chain verification.

## Immediate engineering gates

- [x] Onboarding public Freighter connection on main customer website; Testnet-only public read
- [x] Read-only G/C address parsing and bounded local recipient/amount planning
- [ ] Independently verified smart-wallet factory, verifier, relayer and multi-device recovery on Testnet
- [ ] Selected audited Confidential Token/Testnet asset implementation, proof verification and issuer policy
- [ ] Selected reviewed Private Payments pool implementation with test vectors and nullifier/replay tests
- [ ] Integration tests across SDK, backend, wallet, contract, journal, provider and fee vault
- [ ] Production release signoff for payment, privacy and fee-collection capabilities

**Security rule:** public website, the SDK and contract release flags remain fail-closed. The user may connect a wallet and inspect public data today; they cannot send a confidential payment using StealthBridge yet.

### Recipient contract preflight (public presence only)

Send and Business must not treat a checksum-valid `C…` recipient as an active
contract. The visitor may explicitly request `GET /v1/wallet/contracts/{address}`;
only a live, exact-address, TTL-checked Testnet instance observation is sufficient
to continue **local draft review**. A missing, stale, substituted or unobserved
instance blocks that review. Neither this check nor an Ed25519 signature verifies
whether the recipient can accept the selected asset or authorize settlement.
The isolated restricted-account research now includes a signed nonempty-argument
rejection case. No new contract has been deployed; execution remains disabled.
