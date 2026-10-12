# Privacy primitive verification release gates

Addresses a subset of #11 and #9. These are offline release-preflight requirements, not proof that any privacy mechanism works.

Before a future review can consider changing a capability to true, collect independently reproducible Testnet test vectors, pinned upstream code and build hashes, observer disclosure evidence, adversarial proof and replay tests, cross-wallet interoperability results, issuer authority review, and external security audit evidence. Every requirement must be independently verified.

The checked-in Testnet manifest must remain `not-deployed`, `verified: false`, with no addresses, issuers or transaction hashes until a separate authorized evidence-backed review. The offline release guard rejects unsupported claims.

Run `python3 scripts/verify_release_gates.py` and `PYTHONPATH=scripts python3 -m unittest scripts/test_verify_release_gates.py`. Neither command deploys, signs or transfers anything.
