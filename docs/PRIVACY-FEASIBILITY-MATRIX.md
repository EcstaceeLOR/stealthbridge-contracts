# Privacy primitive feasibility matrix — Research checkpoint (2026-10-08)

Evidence labels:
- **Documented upstream** = source describes behavior; not yet reproduced by StealthBridge.
- **Implemented by StealthBridge** = local demo source exists, not a cryptographic proof.
- **Verified by StealthBridge** = actual testnet transaction hash and assertion artifacts required. **No payment rail meets this status yet.**

| Question | Confidential Tokens | Stellar Private Payments |
| --- | --- | --- |
| Intended privacy | Amount + balance commitments | Amount + counterparties of private transfer |
| Public relationship | Account A → B visible | Private hop unlinkability target, but deposits/withdrawals public |
| Primary fit | B2B known-institution stablecoin transfers | Consumer remittance/receiver note journey |
| Status | Developer preview; testnet research | Developer preview alpha; testnet research |
| Asset support | Issuer/token deployment dependent | Upstream preview documentation advertises XLM and EURC testnet pools; **not an assumed USDC pool** |
| User experience | Wallet signatures, ZK proofs and issuer controls need testing | Prebuilt WASM TypeScript SDK plus Freighter signing; note storage/sync and recovery testing needed |
| Composability | Evaluate issuer-managed policy and custom Soroban integration | Evaluate gateway/recipient/withdraw semantics; do not assume arbitrary cross-contract transfers |
| Auditability | Scope issuer restrictions, viewing authorization and disclosure keys | Pool admission and optional selective disclosure are separate integration concerns |
| Transaction evidence | None yet | None yet |
| Go/no-go | Demonstrate transfer and a truthful ledger observation matrix | Demonstrate deposit → transfer → withdrawal and record public endpoints/timing |

## Source observations
- Official SPP preview: https://stellar.org/blog/developers/developer-preview-stellar-private-payments — XLM/EURC preview pools and \`stellar-private-payments@alpha\` TS/JS package; deposits and withdrawals expose external amount while private transfer has ext_amount zero.
- Issuer-controlled confidentiality essay: https://stellar.org/blog/developers/practical-confidential-stablecoins-an-issuer-controlled-architecture — issuer permissions, restrictions, scoped disclosure and proofs; no claim that a particular issuer has integrated.
- OpenZeppelin Confidential Tokens official blog: https://stellar.org/blog/developers/developer-preview-confidential-tokens-on-stellar
- Stellar ZK skill: https://skills.stellar.org/skills/zk-proofs/SKILL.md

## First POC acceptance
1. Pin SDK/protocol versions and deployed source code reference.
2. Run transfer with fresh testnet-funded accounts, only after approval to create/fund keys.
3. Record exact asset IDs, contract addresses and transaction hashes.
4. Compare expected secrecy against observer ledger, contract events, RPC values, asset history, timing and wallet side channels.
5. Independently test double spend, revoked issuer participant, missing trustline, wallet rejection and device note recovery.
6. State honestly whether the primitive supports our multi-provider corridor contract without leaks or off-chain trust assumptions.

## Issuer-controlled business confidentiality research checkpoint (2026-10-13)

See [ADR-006](ADR-006-ISSUER-CONFIDENTIALITY.md) and its [machine-readable capability/disclosure review](../integrations/confidential-tokens/capability-review.v1.json), pinned to OpenZeppelin `stellar-contracts` commit `6b2f03402b6b98fad6b4247b5e15dce91a0b205d` (workspace 0.7.1; Soroban SDK 28.0.0). This differs from the local registry's Soroban SDK 27.0.6, so compiling our registries is not proof that upstream Confidential Tokens can be composed with them.

**Critical issuer boundary:** wrapping an issuer's transparent SEP-41/SAC token is not issuer authorization to mint, burn, redeem or freeze. Auditor keys and viewing scope, optional policy hooks, reserve backing and off-chain regulated payouts have distinct owners and failure modes. In-wrapped transfer amounts may be confidential while counterparties remain visible and deposit/withdraw amounts public. Nothing has passed StealthBridge's independent cryptographic Testnet reproductions; all payment/privacy capability flags remain false. Unreproduced transaction hashes, contract IDs or providers must not be fabricated.

## Executable *gate inventory*, not executed cryptographic proofs (2026-10-13)

Both rails now have twelve [machine-checked privacy evidence case definitions](../integrations/privacy-evidence-vectors.v1.json) and independent [Confidential Token](../integrations/confidential-tokens/VERIFICATION-GATES.md) / [Private Payments](../integrations/stellar-private-payments/VERIFICATION-GATES.md) acceptance runbooks. CI checks that all six categories (cryptographic soundness, independent audit, two-wallet reproduction, disclosure observation, chain receipts and issuer/pool authorization) remain **unverified**. A future, separately reviewed validator upgrade must consume actual reproducible proof vectors, immutable deployment identifiers and independently inspected transaction receipts. Static test-case descriptions alone do not establish any cryptographic property or live private rail, and no local private witness appears in CI.
