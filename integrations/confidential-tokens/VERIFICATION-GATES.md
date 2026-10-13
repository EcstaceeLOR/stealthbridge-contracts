# Confidential Token verification gate — **NOT VERIFIED**

**Source pin:** [OpenZeppelin `stellar-contracts` `6b2f03402b6b98fad6b4247b5e15dce91a0b205d`](https://github.com/OpenZeppelin/stellar-contracts/tree/6b2f03402b6b98fad6b4247b5e15dce91a0b205d), workspace 0.7.1 / Soroban SDK 28.0.0, upstream Testnet CLI example 28.1.0 and protocol 29. The local StealthBridge registry remains on SDK 27.0.6 and target protocol 27. No executable compatibility or confidential-token integration has been verified.

This track targets **hidden transfer amounts and balances with public sender and receiver identities**. It is **not** an anonymity pool. Proof system: Noir/UltraHonk with Grumpkin commitments; issuer mint/burn and redemption obligations remain outside the wrapper.

## Deterministic test-case contract

Read the six named `ct-*` input/output specifications in [privacy-evidence-vectors.v1.json](../privacy-evidence-vectors.v1.json). The current file specifies the fixtures and required pass/fail decisions, **not cryptographic proof bytes or actual on-chain reproductions**. Before marking `confidential_token_verified` true, each case must be implemented with upstream-pinned witness and public-input fixtures:

- valid two-wallet deposit/transfer/withdraw proves consistent commitment balance and recipient receipt while public observer **still sees sender/receiver identities**;
- tampered proof/public inputs, wrong VK, reused proof, frozen or blocked account all fail **without state mutation**;
- auditor/public-view tests document exactly what holder, authorized auditor, untrusted observer and SAC issuer can decrypt or link;
- transparent deposit/withdraw amounts and account addresses are observed and **not** described as confidential;
- independently verify actual transaction hashes, installed WASM and UltraHonk key provenance across at least two wallets with approved **Testnet-only** accounts; perform a separate security review.

**Pass criteria:** independent verifier replays cryptographic proofs, Rust/Soroban tests and Testnet RPC receipts for all cases, preserves auditor/issuer separation and records exact anonymized observation matrix plus independent reviewer approvals. **Fail criteria:** mismatched circuit/source hash, proof accepted with wrong VK, wrong account/frozen bypass, money conservation failure, public amount leakage beyond the reviewed model, unsupported protocol or missing operator/issuer authorization. Refuse to infer a successful fiat payout from any on-chain transaction.

**Current outcome:** no Testnet hashes/witnesses/receipts, no verified issuer, no live privacy transfers, payment flag false. See [issuer ADR](../../docs/ADR-006-ISSUER-CONFIDENTIALITY.md). Never store private witnesses in GitHub, the backend or server analytics.
