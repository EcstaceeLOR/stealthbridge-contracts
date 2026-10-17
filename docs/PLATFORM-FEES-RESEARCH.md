# Proposed StealthBridge platform fees: staged rollout

> **Not implemented on-chain.** The three deployed governance contracts on
> [Stellar Testnet](../deployments/testnet/VERIFIED-GOVERNANCE-RELEASE-2026-10-17.md)
> do not custody, transfer or collect token assets. Network transaction fees
> are paid to the Stellar network; they are not StealthBridge revenue.

## Product intent

StealthBridge may eventually charge a small, visible fee only for a
**successfully executed transaction through a specifically supported rail**.
An illustrative SDK **quote** starts at 10 basis points (0.10%), capped at
50 basis points (0.50%). This is a *proposal*, not an active charge or
a promise that any transfer rail is operational.

Quotes are inclusive: for a 100-unit token payment at 10 bps,
the recipient is quoted 99.9 and the platform 0.1 (subject to the
asset's exact minor-unit precision). Fee calculation rounds down to
avoid surprise minimum charges. The payer must be shown both amounts
and authorize the exact transfer before execution.

## Contract architecture, still requiring implementation

Keep these three reviewed governance contracts **unchanged**:

- `CorridorRegistry` — public corridor configuration and pause.
- `PolicyRegistry` — public policy commitments and revisions.
- `GovernanceGate` — read-only composition of public flags.

A **separate FeeCollector / SettlementAdapter** may later:

1. Restrict accepted assets to explicitly allowlisted, reviewed **SEP-41**
   token-contract C-addresses and verified decimals/issuer capabilities.
2. Require a payer-authorized exact gross amount, network, asset, target
   recipient, fee cap, fee treasury, quote expiry, and idempotency key.
3. Atomically transfer the fee and net amount in the same reviewed
   transaction, or revert everything; never charge for a failed transfer.
4. Restrict any rate mutation to authorized governance with a hard upper
   bound, delay, pause, treasury rotation and events. Use carefully scoped
   capability controls, not a deployer's hot-key wildcard allowance.
5. Keep fee treasury in a separately governed, auditable account.
   It need not be the same as `operator_local` or the administrator.
6. Model replay protection, TTL, restoration, accounting reconciliation,
   token approval semantics, dust amounts, rounding, stale quotes,
   authorization footprint and abuse-rate limits in tests.
7. Only use a **Testnet sandbox token** while this is unaudited.
   Production use requires independent security review, issuer
   agreements, legal/tax/regulatory analysis and proper custody controls.

Transparent Stellar token transfers disclose asset, contract, amount and
other public metadata. Do not describe such a fee contract as private:
a future confidential rail has different proof, disclosure and
compliance requirements that must be reviewed separately.

## SDK / UI release boundaries

The SDK's `quotePlatformFee` is a pure, fixed-input bigint computation.
It returns `collectionEnabled: false` and cannot send a transaction.
The frontend must not imply that fee collection is live. A dedicated fee
contract would require a new source/ABI, independent Testnet deployment,
full on-chain attestation and a separately permissioned SDK signing adapter.

The final business rate and any Mainnet treasury should be chosen only
after payment-rail validation, fee disclosure and an explicit release
approval. Neither this design document nor any registry's public flag
automatically enables a fund-moving feature.
