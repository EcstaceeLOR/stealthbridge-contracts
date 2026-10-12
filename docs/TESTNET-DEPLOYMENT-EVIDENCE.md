# Testnet deployment evidence chain and recovery runbook

**Status: source-only, NOT DEPLOYED.** No account authorization, Soroban activation, confidential transfer or payment execution is claimed. This document defines review gates, not permission to sign or deploy.

## Sequence and independent evidence

Each proposed registry (corridor, policy and governance-gate) must have independently reproducible release evidence before reviewers can authorize a *separate* deployment change:

1. **Source reproducibility:** pin reviewed source commit, Rust toolchain, Cargo.lock SHA-256, pinned Stellar CLI, exact protocol and target. Build twice in independent directories; retain both WASM digests, CLI-extracted ABIs, artifact manifest and CI run URL.
2. **Operator authorization:** bind the authorized Testnet G-address, governance approval context, chain passphrase, intended contract IDs and bytecode hashes in an operator-signed release statement. Verify the actual Stellar signature using an independently reviewed public-key verifier; syntax or a pasted signature string alone is **not sufficient**.
3. **Testnet transaction inclusion:** independently query `getTransaction` and ledger data for actual deployment transaction hashes. Verify transaction status, ledger sequence, network, authorized source and immutable contract ID. Two independent observers must agree; don't mistake simulation for execution.
4. **On-chain WASM/interface parity:** independently retrieve the installed contract executable/hash from Testnet and match the locally reproduced WASM plus all public methods and arguments. Check all three registry IDs and explicit version transitions.
5. **Pause and recovery drill:** prove the authorized emergency pause blocks enabling policy/corridors, retain the previously reviewed WASM, prepare a reversible upgrade path (or a replacement deployment plan when rollback is impossible), model storage TTL/restoration costs, and record an authorized paused-to-restored test.
6. **Independent review:** a different operator or security reviewer replays all RPC/evidence checks and approves the exact record; required disclosure/security controls and separation of duties must be documented.

**Artifacts:** retain the complete signed statement, source commit, both WASM and ABI digests, proof of signature validation, on-chain transaction IDs and RPC receipt bytes or SHA-256 digests, reconstructed account/contract IDs, Testnet passphrase, observer confirmations, rollback/pause transaction evidence, reviewers and timestamps in immutable project release storage. Never paste keys, private witnesses or provider credentials into GitHub.

## Fail-closed source-only gate

`deployments/testnet/evidence-chain.v1.json` declares the **six pending** checks, no signatures, transaction hashes, contract addresses, evidence receipts or financial capabilities. CI runs:

```bash
python3 scripts/validate_deployment_evidence.py
python3 -m unittest scripts/test_validate_deployment_evidence.py
python3 scripts/validate_manifest.py
```

The validator rejects a populated deployment record, positive privacy flag, forgery of an operator signature, wrong network, incomplete gate, conflicting manifest and unsupported contract identity. It does **not** perform RPC or signature verification. An actual deployment requires a **reviewed v2 evidence schema and validator** that cryptographically and independently verifies all external evidence before any manifest status or financial capability may change. No shortcut to declaring verified is available in the v1 source-only gate.

## Emergency stop and rollback

For an affected registry, authorized operators should first **pause** the affected public policy/corridor if the current contract permits it. This is not a private-token freeze or settlement reversal. Record before/after Testnet RPC states, authorization, ledger and exact tx hashes. For an upgrade, require the contract's hash-locked proposal, delay, expiry and pause invariants; execute a reviewed recovery WASM only through the approved authorization path, then check the installed executable and state on-chain. If upgrade or rollback is impossible, leave paused, preserve evidence and plan a new deployment with new IDs and explicit backend/SDK consumers' discovery-version migration. Never claim a completed financial reversal or assume an emitted event is proof of transfer confidentiality.

**Rollback cannot undo prior immutable ledger transactions, confidential transfers, or off-chain fiat payouts.** Full recovery requires independent security signoff, observer reconciliation and partner/legal handling where applicable. No automatic reactivation follows a code or CI success.
