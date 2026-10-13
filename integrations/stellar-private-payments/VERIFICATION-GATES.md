# Stellar Private Payments verification gate — **NOT VERIFIED**

**Source pin:** [NethermindEth `stellar-private-payments` `0471e6be8161c2013cfc8387bd7b72b70e480cf5`](https://github.com/NethermindEth/stellar-private-payments/tree/0471e6be8161c2013cfc8387bd7b72b70e480cf5). The alpha is evolving, unaudited and can change contracts, pools, indexers and client state; do not infer a deployed pool ID, asset, on-chain guarantee or protocol compatibility from a blog/demo. Proof system: Groth16, client-owned note/witness storage.

The intended private-pool flow is deposit → shielded transfer → withdraw with private internal counterparty/amount and **public deposit/withdraw edges, asset, timing and transfer presence**. Pool association-set providers (ASP) and regulated provider onboarding are distinct controls. Do not equate this rail with amount-only Confidential Tokens.

## Deterministic test-case contract

Read the six `spp-*` input/output specifications in [privacy-evidence-vectors.v1.json](../privacy-evidence-vectors.v1.json); these are required **test-case schemas**, **not fabricated executable ZK proof vectors**. Before enabling `private_payments_verified`:

- initialize two separately controlled wallet-owned Testnet clients and record version, circuits, proving key, pool contract ID and **verified** transaction receipts;
- independently observe a deposit, shielded transfer to another wallet, and withdrawal; verify public external amount edges and internal private account relationships against a ledger observer;
- reject replayed nullifier, forged ASP membership, invalid withdraw amount and a wrong-chain proof with unchanged state; prove no successful repeated redemption;
- demonstrate client-storage loss, recovery, wrong recipient, expired proof, wallet approval denial and untrusted indexer behavior without invented private balances;
- observe wallet/indexer/network disclosures (including timestamp correlation) and a compliant association-set policy;
- record independent auditor/security review and prove integration boundary with any StealthBridge custom Soroban registry **without claiming an unsupported private contract-to-contract transfer**.

**Pass criteria:** reproducible, independently reviewed deposit/transfer/withdraw transactions with exact source/proof versions, two wallets, immutable receipts and honest disclosure report. **Fail criteria:** missing contract provenance, nonce/nullifier replay acceptance, public secret disclosure, data loss misreported as success, unknown destination binding or unverified payout provider.

**Current outcome:** no signed proof, wallet run, verified pool ID, transactions, deployment or payout. Funding, secret keys and on-chain writes require explicit separately authorized Testnet-only action; the source-only CI does none of them.
