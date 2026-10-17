# Private Business Reviews vs Public Soroban State

StealthBridge business review requests and maker-checker memberships remain off-chain, protected by authenticated PostgreSQL tenant scopes. No organization session secret, reviewer identity, review database row, wallet login challenge, private proof witness or fiat routing arrangement belongs in a public Soroban registry.

`tools/release-boundary-audit.rs` enforces source gates against reintroducing settlement functions or leaking enterprise authentication/review storage into the public corridor, policy and governance contracts. CI also requires safe undeployed Testnet manifest evidence.

These source audits are **not** an independent security audit, zero-knowledge proof correctness test, deployment attestation or authorization to release payment execution. Any future contract design should support independently reviewed upgrades, on-chain policy commitments **without private identities**, bounded resource use and a separate mainnet release process.

There is no deployed or independently verified confidential payment rail in this stage. Business reviewer counts are not Soroban transfer authorizations.
