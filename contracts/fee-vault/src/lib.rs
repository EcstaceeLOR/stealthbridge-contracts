#![no_std]
//! Isolated transparent fee-vault prototype. No private payments or automatic settlement.
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, token, Address, Env};
const TTL_THRESHOLD: u32 = 17_280;
const TTL_EXTEND: u32 = 30 * TTL_THRESHOLD;

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    PendingAdmin,
    Asset,
    Paused,
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FeeVaultError {
    InvalidAmount = 1,
    Paused = 2,
    NoPendingAdmin = 3,
    InsufficientBalance = 4,
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
fn asset(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Asset).unwrap()
}
fn renew(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND);
}

#[contractimpl]
impl FeeVault {
    /// A nominated administrator MUST sign deployment. Token is immutable.
    pub fn __constructor(env: Env, administrator: Address, token_address: Address) {
        administrator.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::Admin, &administrator);
        env.storage()
            .instance()
            .set(&DataKey::Asset, &token_address);
        env.storage().instance().set(&DataKey::Paused, &true);
        renew(&env);
    }
    pub fn get_admin(env: Env) -> Address {
        admin(&env)
    }
    pub fn get_asset(env: Env) -> Address {
        asset(&env)
    }
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(true)
    }
    pub fn set_paused(env: Env, paused: bool) {
        require_admin(&env);
        env.storage().instance().set(&DataKey::Paused, &paused);
        renew(&env);
    }

    /// User-consented transfer of the configured SEP-41 asset, NOT principal.
    /// Token transfers reveal sender, amount, and vault on the public ledger.
    pub fn deposit_fee(env: Env, payer: Address, amount: i128) -> Result<(), FeeVaultError> {
        if Self::is_paused(env.clone()) {
            return Err(FeeVaultError::Paused);
        }
        if amount <= 0 {
            return Err(FeeVaultError::InvalidAmount);
        }
        payer.require_auth();
        token::Client::new(&env, &asset(&env)).transfer(
            &payer,
            &env.current_contract_address(),
            &amount,
        );
        renew(&env);
        Ok(())
    }

    /// Actual public token balance; may include unsolicited direct transfers.
    pub fn vault_balance(env: Env) -> i128 {
        token::Client::new(&env, &asset(&env)).balance(&env.current_contract_address())
    }

    /// ONLY active administrator may withdraw; destination is never caller-supplied.
    /// Allowed during emergency pause for incident recovery.
    pub fn claim_fees(env: Env, amount: i128) -> Result<(), FeeVaultError> {
        let recipient = require_admin(&env);
        if amount <= 0 {
            return Err(FeeVaultError::InvalidAmount);
        }
        let client = token::Client::new(&env, &asset(&env));
        if amount > client.balance(&env.current_contract_address()) {
            return Err(FeeVaultError::InsufficientBalance);
        }
        client.transfer(&env.current_contract_address(), &recipient, &amount);
        renew(&env);
        Ok(())
    }
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
        env.storage().instance().set(&DataKey::Admin, &successor);
        env.storage().instance().remove(&DataKey::PendingAdmin);
        renew(&env);
        Ok(())
    }
}
#[cfg(test)]
mod tests;
