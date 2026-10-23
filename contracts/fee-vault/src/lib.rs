#![no_std]
//! Isolated transparent fee-vault research. No privacy, settlement or automatic collection.
//! Neither the governing registries nor this vault authorize customer payments.
use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, token, Address, BytesN, Env,
};
const TTL_THRESHOLD: u32 = 17_280;
const TTL_EXTEND: u32 = 30 * TTL_THRESHOLD;

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    PendingAdmin,
    Beneficiary,
    PendingBeneficiary,
    Asset,
    Paused,
    ClaimsPaused,
    Claimable,
    TransferInProgress,
    Upgrade,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeProposal {
    pub wasm_hash: BytesN<32>,
    pub ready_at_ledger: u32,
    pub expires_at_ledger: u32,
}

#[contractevent(topics = ["vault", "deposit"])]
#[derive(Clone)]
pub struct FeeDeposited {
    pub payer: Address,
    pub amount: i128,
}

#[contractevent(topics = ["vault", "claim"])]
#[derive(Clone)]
pub struct FeeClaimed {
    pub beneficiary: Address,
    pub amount: i128,
}

#[contractevent(topics = ["vault", "admin"])]
#[derive(Clone)]
pub struct AdminRotated {
    pub administrator: Address,
}

#[contractevent(topics = ["vault", "beneficiary"])]
#[derive(Clone)]
pub struct BeneficiaryRotated {
    pub beneficiary: Address,
}

#[contractevent(topics = ["vault", "upgrade_staged"])]
#[derive(Clone)]
pub struct UpgradeStaged {
    pub wasm_hash: BytesN<32>,
    pub ready_at_ledger: u32,
}

#[contractevent(topics = ["vault", "upgrade_cancelled"])]
#[derive(Clone)]
pub struct UpgradeCancelled {
    pub wasm_hash: BytesN<32>,
}

#[contractevent(topics = ["vault", "upgrade_applied"])]
#[derive(Clone)]
pub struct UpgradeApplied {
    pub wasm_hash: BytesN<32>,
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FeeVaultError {
    InvalidAmount = 1,
    Paused = 2,
    NoPendingAdmin = 3,
    InsufficientBalance = 4,
    ArithmeticOverflow = 5,
    ReentrantTransfer = 6,
    NoPendingBeneficiary = 7,
    UpgradeRequiresPause = 8,
    InvalidUpgradeHash = 9,
    InvalidUpgradeDelay = 10,
    NoPendingUpgrade = 11,
    UpgradeNotReady = 12,
    UpgradeExpired = 13,
    ClaimsPaused = 14,
}

#[contract]
pub struct FeeVault;

fn admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

fn require_admin(env: &Env) -> Address {
    let current = admin(env);
    current.require_auth();
    current
}

fn beneficiary(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Beneficiary).unwrap()
}

fn asset(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Asset).unwrap()
}

fn renew(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND);
}

fn lock_transfer(env: &Env) -> Result<(), FeeVaultError> {
    if env
        .storage()
        .instance()
        .get::<_, bool>(&DataKey::TransferInProgress)
        .unwrap_or(false)
    {
        return Err(FeeVaultError::ReentrantTransfer);
    }
    env.storage()
        .instance()
        .set(&DataKey::TransferInProgress, &true);
    Ok(())
}

fn unlock_transfer(env: &Env) {
    env.storage()
        .instance()
        .set(&DataKey::TransferInProgress, &false);
}

fn claimable(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::Claimable)
        .unwrap_or(0)
}

#[contractimpl]
impl FeeVault {
    /// Asset is immutable. Admin must authorize construction and initially
    /// holds the explicit beneficiary role; these roles can diverge on rotation.
    pub fn __constructor(env: Env, administrator: Address, token_address: Address) {
        administrator.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::Admin, &administrator);
        env.storage()
            .instance()
            .set(&DataKey::Beneficiary, &administrator);
        env.storage()
            .instance()
            .set(&DataKey::Asset, &token_address);
        env.storage()
            .instance()
            .set(&DataKey::Paused, &true);
        env.storage()
            .instance()
            .set(&DataKey::ClaimsPaused, &true);
        env.storage()
            .instance()
            .set(&DataKey::Claimable, &0_i128);
        renew(&env);
    }
    pub fn get_admin(env: Env) -> Address {
        admin(&env)
    }
    pub fn get_beneficiary(env: Env) -> Address {
        beneficiary(&env)
    }
    pub fn get_asset(env: Env) -> Address {
        asset(&env)
    }
    pub fn claimable_fees(env: Env) -> i128 {
        claimable(&env)
    }
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(true)
    }
    pub fn set_paused(env: Env, paused: bool) {
        require_admin(&env);
        env.storage()
            .instance()
            .set(&DataKey::Paused, &paused);
        renew(&env);
    }

    /// Claims are independently disabled by default, including during deposit pause.
    /// Explicit admin authorization is required to unblock existing fee claims.
    pub fn claims_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::ClaimsPaused)
            .unwrap_or(true)
    }
    pub fn set_claims_paused(env: Env, paused: bool) {
        require_admin(&env);
        env.storage()
            .instance()
            .set(&DataKey::ClaimsPaused, &paused);
        renew(&env);
    }

    /// Explicit, separately authorized transparent SEP-41 token deposit.
    /// Unsolicited token transfers NEVER increase accounted claimable fees.
    pub fn deposit_fee(env: Env, payer: Address, amount: i128) -> Result<(), FeeVaultError> {
        if Self::is_paused(env.clone()) {
            return Err(FeeVaultError::Paused);
        }
        if amount <= 0 {
            return Err(FeeVaultError::InvalidAmount);
        }
        payer.require_auth();
        let next = claimable(&env)
            .checked_add(amount)
            .ok_or(FeeVaultError::ArithmeticOverflow)?;
        lock_transfer(&env)?;
        token::Client::new(&env, &asset(&env)).transfer(
            &payer,
            &env.current_contract_address(),
            &amount,
        );
        env.storage()
            .instance()
            .set(&DataKey::Claimable, &next);
        unlock_transfer(&env);
        FeeDeposited { payer, amount }.publish(&env);
        renew(&env);
        Ok(())
    }

    /// Public balance includes unsolicited transfers and is NOT accrued revenue.
    pub fn vault_balance(env: Env) -> i128 {
        token::Client::new(&env, &asset(&env)).balance(&env.current_contract_address())
    }

    /// Claimable ledger is debited BEFORE the external token call. Soroban
    /// reverts the entire invocation if that call traps or returns an error.
    /// Beneficiary must authorize; caller cannot provide a payout destination.
    /// Claim pause is independent of deposit pause and initially ENABLED;
    /// claims can be unlocked only by an authorized administrator.
    pub fn claim_fees(env: Env, amount: i128) -> Result<(), FeeVaultError> {
        beneficiary(&env).require_auth();
        if amount <= 0 {
            return Err(FeeVaultError::InvalidAmount);
        }
        if Self::claims_paused(env.clone()) {
            return Err(FeeVaultError::ClaimsPaused);
        }
        if amount > claimable(&env) {
            return Err(FeeVaultError::InsufficientBalance);
        }
        // Enter the reentrancy guard BEFORE the first external token call,
        // including the balance check (a malicious token may call us back).
        lock_transfer(&env)?;
        let client = token::Client::new(&env, &asset(&env));
        if amount > client.balance(&env.current_contract_address()) {
            unlock_transfer(&env);
            return Err(FeeVaultError::InsufficientBalance);
        }
        env.storage()
            .instance()
            .set(&DataKey::Claimable, &(claimable(&env) - amount));
        let recipient = beneficiary(&env);
        client.transfer(&env.current_contract_address(), &recipient, &amount);
        unlock_transfer(&env);
        FeeClaimed {
            beneficiary: recipient,
            amount,
        }
        .publish(&env);
        renew(&env);
        Ok(())
    }

    /// Two-step administrator rotation. Use a recoverable Soroban multisig
    /// contract/account as administrator, not a single local CLI alias.
    pub fn propose_admin(env: Env, successor: Address) {
        require_admin(&env);
        env.storage()
            .instance()
            .set(&DataKey::PendingAdmin, &successor);
        renew(&env);
    }
    pub fn pending_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::PendingAdmin)
    }
    pub fn cancel_admin_proposal(env: Env) {
        require_admin(&env);
        env.storage().instance().remove(&DataKey::PendingAdmin);
        renew(&env);
    }
    pub fn accept_admin(env: Env) -> Result<(), FeeVaultError> {
        let successor: Address = env
            .storage()
            .instance()
            .get(&DataKey::PendingAdmin)
            .ok_or(FeeVaultError::NoPendingAdmin)?;
        successor.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::Admin, &successor);
        env.storage().instance().remove(&DataKey::PendingAdmin);
        AdminRotated {
            administrator: successor,
        }
        .publish(&env);
        renew(&env);
        Ok(())
    }

    /// Beneficiary requires separate nomination and acceptance. The admin may
    /// nominate a recovery beneficiary, but cannot claim as that beneficiary.
    pub fn propose_beneficiary(env: Env, successor: Address) {
        require_admin(&env);
        env.storage()
            .instance()
            .set(&DataKey::PendingBeneficiary, &successor);
        renew(&env);
    }
    pub fn pending_beneficiary(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::PendingBeneficiary)
    }
    pub fn cancel_beneficiary_proposal(env: Env) {
        require_admin(&env);
        env.storage().instance().remove(&DataKey::PendingBeneficiary);
        renew(&env);
    }
    pub fn accept_beneficiary(env: Env) -> Result<(), FeeVaultError> {
        let successor: Address = env
            .storage()
            .instance()
            .get(&DataKey::PendingBeneficiary)
            .ok_or(FeeVaultError::NoPendingBeneficiary)?;
        successor.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::Beneficiary, &successor);
        env.storage().instance().remove(&DataKey::PendingBeneficiary);
        BeneficiaryRotated {
            beneficiary: successor,
        }
        .publish(&env);
        renew(&env);
        Ok(())
    }

    /// Hash-pinned delayed upgrade. This is still a single-role release policy;
    /// production requires independent multisig approval and code audit.
    pub fn propose_upgrade(
        env: Env,
        wasm_hash: BytesN<32>,
        delay_ledgers: u32,
    ) -> Result<(), FeeVaultError> {
        require_admin(&env);
        if !Self::is_paused(env.clone()) {
            return Err(FeeVaultError::UpgradeRequiresPause);
        }
        if wasm_hash == BytesN::from_array(&env, &[0; 32]) {
            return Err(FeeVaultError::InvalidUpgradeHash);
        }
        if !(5..=20_000).contains(&delay_ledgers) {
            return Err(FeeVaultError::InvalidUpgradeDelay);
        }
        let ready = env
            .ledger()
            .sequence()
            .checked_add(delay_ledgers)
            .ok_or(FeeVaultError::InvalidUpgradeDelay)?;
        let expires = ready
            .checked_add(10_000)
            .ok_or(FeeVaultError::InvalidUpgradeDelay)?;
        env.storage().instance().set(
            &DataKey::Upgrade,
            &UpgradeProposal {
                wasm_hash: wasm_hash.clone(),
                ready_at_ledger: ready,
                expires_at_ledger: expires,
            },
        );
        UpgradeStaged {
            wasm_hash,
            ready_at_ledger: ready,
        }
        .publish(&env);
        renew(&env);
        Ok(())
    }
    pub fn pending_upgrade(env: Env) -> Option<UpgradeProposal> {
        env.storage().instance().get(&DataKey::Upgrade)
    }
    pub fn cancel_upgrade(env: Env) {
        require_admin(&env);
        let pending: Option<UpgradeProposal> = env.storage().instance().get(&DataKey::Upgrade);
        env.storage().instance().remove(&DataKey::Upgrade);
        if let Some(pending) = pending {
            UpgradeCancelled {
                wasm_hash: pending.wasm_hash,
            }
            .publish(&env);
        }
        renew(&env);
    }
    pub fn execute_upgrade(env: Env, wasm_hash: BytesN<32>) -> Result<(), FeeVaultError> {
        require_admin(&env);
        if !Self::is_paused(env.clone()) {
            return Err(FeeVaultError::UpgradeRequiresPause);
        }
        let pending: UpgradeProposal = env
            .storage()
            .instance()
            .get(&DataKey::Upgrade)
            .ok_or(FeeVaultError::NoPendingUpgrade)?;
        if pending.wasm_hash != wasm_hash {
            return Err(FeeVaultError::InvalidUpgradeHash);
        }
        let now = env.ledger().sequence();
        if now < pending.ready_at_ledger {
            return Err(FeeVaultError::UpgradeNotReady);
        }
        if now > pending.expires_at_ledger {
            return Err(FeeVaultError::UpgradeExpired);
        }
        env.storage().instance().remove(&DataKey::Upgrade);
        renew(&env);
        UpgradeApplied {
            wasm_hash: wasm_hash.clone(),
        }
        .publish(&env);
        env.deployer().update_current_contract_wasm(wasm_hash);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
