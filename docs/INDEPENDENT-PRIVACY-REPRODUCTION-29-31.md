# Issues #29 and #31 — independent privacy reproduction checklist

**Current status: NOT VERIFIED / NO_GO.** This PR does not establish amount
confidentiality, relationship unlinkability, ZK soundness, or a deployed
private settlement rail. Governance deployment is not a payment test.

## Two distinct candidate mechanisms

| Candidate | Intended confidentiality | Actual test to perform | Known visible surfaces |
| --- | --- | --- | --- |
| Confidential Tokens / UltraHonk | Transfer amount within supported issuer-controlled wrapper; not counterparties | Independently rebuild pinned upstream source, verify proof/VK hashes, mint only with issuer authorization, transfer from two real Testnet wallets, mutate commitment/public input/VK, test frozen account and replay | Accounts, transparent deposit/withdraw edges, token contract identifiers, event timing |
| Stellar Private Payments / Groth16 | Shielded amount and counterparty within a supported pool | Pin circuits/VK/WASM, deposit/transfer/withdraw with two independent Testnet wallets; reuse nullifier, forge membership/root, corrupt proof/withdrawal and restore notes on another device | Deposit/withdraw amount and address, network fees, timing, pool/relayer/operator metadata |

Upstream checks and prewritten test vectors in
`integrations/privacy-evidence-vectors.v1.json` are **hypotheses**, not passing
cryptographic test vectors. No private witness or proof key may enter GitHub,
CI logs, sample outputs or issue comments.

## Reviewable evidence collection (requires independent authorized operator)

1. Identify the exact Testnet protocol, consensus network passphrase, source
   commit, toolchains, circuit description and verifier key SHA-256 hash.
   Check underlying SDK protocol compatibility independently.
2. For each test vector, use a fresh authorized Testnet wallet, approved
   test-only asset/pool, recorded input/public-output mapping, and a public
   transaction hash. Record the exact deployed *contract ID*, WASM SHA-256
   (from independently fetched on-chain executable), sanitized receipt,
   ledger number and negative witness test category. Store secrets offline.
3. Inspect the same transactions through a **second** network RPC and
   independently compare returned ledger/status hashes. Run, for example:

       python3 scripts/verify_privacy_testnet_receipts.py --hash <PUBLIC_64_HEX_TX_HASH> --expected-status SUCCESS --rpc-url https://<FIRST_TESTNET_RPC> --confirm-rpc-url https://<SEPARATE_TESTNET_RPC>

   Use two separately operated RPC endpoints, not two URLs owned by the
   same untrusted provider. This checks matching status and ledger, not
   proof correctness. For a test expected to fail atomically, use
   `--expected-status FAILED`. A FAILED receipt alone does not prove the verifier rejected for the
   intended reason: preserve and review public diagnostics/contract state.
   `getTransaction` statuses are not mathematical proof verification.
4. Reproduce valid proof and invalid variants with independent Rust/Soroban
   host tests and real protocol-version-compatible WASM. Verify the
   transaction actually invoked the intended deployed executable with
   the correct public inputs and verifier key; re-query contract state after
   every negative test. Compare privacy leakage in XDR, events, token edges,
   RPC results, explorer/indexer views, transaction ordering, wallet/network
   metadata, and wallet recovery pathways. A network-success response is
   not a confidentiality guarantee.
5. Require a separate external security reviewer to sign off on both
   candidate mechanisms, threat model, protocol versions, negative proofs,
   replay/nullifier uniqueness, wallet interoperability, authorization and
   metadata leakage. Record signed approval separately per mechanism.
6. Keep `paymentExecutionEnabled: false`, `NO_GO`, and every privacy
   capability flag false until a **new independently audited verifier and
   versioned evidence-ingestion policy** are merged. The current source-only
   release gate intentionally cannot assert GO.

### Remaining blockers for these issues

No locally reproduced ZK circuits and verifier artifacts, public Testnet
transaction receipts, independently computed WASM hashes, cross-wallet
execution, ledger-state negative proofs or signed reviewer verdict have been
supplied. **Do not close #29 or #31 as implemented or merge a claim that the
privacy primitive is verified.** The read-only RPC receipt checker is only
an evidence collection helper and must not turn these blockers into PASS.
