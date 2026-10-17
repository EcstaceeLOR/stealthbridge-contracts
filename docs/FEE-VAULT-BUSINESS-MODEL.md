# StealthBridge fee vault — isolated Testnet research contract

**NOT DEPLOYED. NOT AUDITED. NOT CONNECTED TO PAYMENTS. NO MAINNET RELEASE.**

The existing three Testnet governance contracts are not payment vaults.
This fourth Rust/Soroban crate models only separately consented fee custody
and administrator claims. It is intentionally in a separate Cargo workspace,
so the three verified governance artifacts and their pinned provenance remain
unchanged. Source-only testing does not authorize live fund custody.

## Business model

* The SDK's proposed fee quote is 10 bps (0.10%) by default, with a
  50 bps (0.50%) configurable ceiling. A quote cannot authorize collection.
* Users must see and authorize gross amount, fee, net amount, currency,
  FX disclosure and any other charges before a real payment.
* One FeeVault instance handles one immutable SEP-41 asset per network.
  There is no arbitrary withdrawal token parameter.
* Constructor requires `administrator.require_auth()`. Initially the
  administrator can be `operator_local` public G-address
  `GC3YRVLB7IN7SNUYVWFUC6OD3QWJXCUZ4HOBNIPVGY7BZDVOYMDNX5TN`.
* Vault starts paused. No customer-fee collection is enabled.
* `deposit_fee(payer, amount)` requires payer's authorization and transfers
  explicit token base units, not an undisclosed part of principal payments.
* `claim_fees(amount)` requires the CURRENT administrator's Soroban
  signature and always pays that same administrator address. There is NO
  arbitrary recipient parameter and no backend/admin-key bypass.
* A two-step administrator handover requires nomination by the current
  admin and consented acceptance by the successor.
* Public `vault_balance()` reports actually held tokens, which could
  include unsolicited transfers; it does not prove net revenue or approved
  settlement receipts. Reconciliation needs a separate audited system.
* Public token transfers leak the amount, payer and vault address.
  THIS PROTOTYPE DOES NOT PROVIDE CONFIDENTIAL PAYMENTS OR HIDE FEE ACTIVITY.
  The privacy rail must be independently reviewed before financial use.

## Accessing operator fees without importing into Freighter

The CLI supports your secure-store identity by its alias. After an
independently audited and explicitly authorized Testnet deployment,
and only after fees have been consentedly collected, review CLI help:

    stellar contract invoke --id VERIFIED_FEE_VAULT_C_ADDRESS --network testnet --source-account operator_local -- --help

An eventual claim to the **current on-chain admin address** is:

    stellar contract invoke --id VERIFIED_FEE_VAULT_C_ADDRESS --network testnet --source-account operator_local -- claim_fees --amount AMOUNT_IN_BASE_UNITS

Do not execute the example against real funds now: this contract is still
unaudited research. The amount uses integer token base units, not decimal
display balances. Never paste a secure-store key or recovery phrase to a
website. Secure store + a compatible Stellar CLI is sufficient for operator
signatures; Freighter is not required.

Before actual fee collection: security review, external audit, replay/consent
proofs, asset validation, ledger reconciliation, noncustodial privacy design,
customer disclosures, issuer and legal assessments, multisig and recovery,
independent release attestation, Testnet end-to-end scenarios and Mainnet
change management. Testnet XLM is not real revenue.
