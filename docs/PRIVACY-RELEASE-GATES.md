# Privacy primitive verification release gates

Addresses a subset of #11 and #9. These are offline release-preflight requirements, not proof that any privacy mechanism works.

Before a future review can consider changing a capability to true, collect independently reproducible Testnet test vectors, pinned upstream code and build hashes, observer disclosure evidence, adversarial proof and replay tests, cross-wallet interoperability results, issuer authority review, and external security audit evidence. Every requirement must be independently verified.

The checked-in Testnet manifest must remain `not-deployed`, `verified: false`, with no addresses, issuers or transaction hashes until a separate authorized evidence-backed review. The offline release guard rejects unsupported claims.

Run `python3 scripts/verify_release_gates.py` and `PYTHONPATH=scripts python3 -m unittest scripts/test_verify_release_gates.py`. Neither command deploys, signs or transfers anything.

## Versioned, fail-closed aggregate release decision

Execute `python3 scripts/attest_privacy_release.py` and `python3 -m unittest scripts/test_attest_privacy_release.py`. The machine-readable output has a **NO_GO** verdict, two distinct rails with independent proof systems, and the six explicit missing approval dimensions (cryptographic soundness, independent audit, cross-wallet reproduction, public disclosure, live ledger receipts and issuer/pool authorization). This single guard validates the immutable upstream source lock, issuer capability matrix, 12 unverified proof-case definitions, source-only Soroban interface and undeployed Testnet manifest in one check.

The source lock passing means **only** that public files at pinned Git commits retain reviewed bytes. Neither that nor green Soroban CI supplies a transferable privacy proof, authorized issuer, deployed pool, wallet-held witness or Testnet transfer receipt. The v1 policy deliberately cannot turn any verification/payment flag true. A future **new policy/schema version** must define secure evidence ingestion, signatures, independent auditors, reproducible circuits/verifier keys and on-chain receipt validation before a separate protocol review can enable execution.

**Contracts issue #11 is about the existence and enforcement of the pre-execution gates, not completion of the evidence.** The actual cryptographic reproduction remains separately tracked under contracts #2 and must remain open. Closing #11 is not authorization to ship transfers or claim private settlement.
