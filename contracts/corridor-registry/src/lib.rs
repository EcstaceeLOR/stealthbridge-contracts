#![no_std]
//! Public corridor enablement flags only: no protected amounts, counterparties,
//! KYC, FX rates, issuer promises or fiat eligibility are stored here.
//!
//! "Enabled" is a governance flag, never a guarantee of available liquidity.

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, BytesN, Env, String,
};

const TTL_THRESHOLD: u32 = 17_280;
const TTL_EXTEND: u32 = 30 * TTL_THRESHOLD;

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    PendingAdmin,
    Paused,
    Corridor(String),
    Approval(String),
    Upgrade,
}

/// Public, on-chain approval of a *specific* off-chain corridor config.
/// A digest is not a verification of a provider, issuer or available funds.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorridorApproval {
    pub config_digest: BytesN<32>,
    pub expires_at_ledger: u32,
}
/// Pending release review; a hash is an identifier, never proof of an audit.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeProposal {
    pub wasm_hash: BytesN<32>,
    pub ready_at_ledger: u32,
    pub expires_at_ledger: u32,
}
/// Public release notice only. Contains no private witness, identity or payment.
#[contractevent(topics = ["upgrade", "staged"])]
#[derive(Clone)]
pub struct UpgradeStaged {
    pub wasm_hash: BytesN<32>,
    pub ready_at_ledger: u32,
    pub expires_at_ledger: u32,
}
#[contractevent(topics = ["upgrade", "cancel"])]
#[derive(Clone)]
pub struct UpgradeCancelled {
    pub wasm_hash: BytesN<32>,
}
#[contractevent(topics = ["upgrade", "applied"])]
#[derive(Clone)]
pub struct UpgradeApplied {
    pub wasm_hash: BytesN<32>,
}
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum RegistryError {
    NotInitialized = 1,
    NoPendingAdmin = 2,
    InvalidCorridorId = 3,
    Paused = 4,
    InvalidExpiration = 5,
    NotEnabled = 6,
    UpgradeRequiresPause = 7,
    InvalidUpgradeDelay = 8,
    InvalidUpgradeHash = 9,
    NoPendingUpgrade = 10,
    UpgradeNotReady = 11,
    UpgradeExpired = 12,
    InvalidConfigDigest = 13,
}

#[contract]
pub struct CorridorRegistry;

fn admin(env: &Env) -> Result<Address, RegistryError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(RegistryError::NotInitialized)
}
fn authorize_admin(env: &Env) -> Result<Address, RegistryError> {
    let current = admin(env)?;
    current.require_auth();
    Ok(current)
}
fn renew_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND);
}

#[contractimpl]
impl CorridorRegistry {
    pub fn __constructor(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        renew_instance(&env);
    }

    /// Public governance address; no payment or policy capabilities implied.
    pub fn get_admin(env: Env) -> Result<Address, RegistryError> {
        admin(&env)
    }

    pub fn pending_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::PendingAdmin)
    }

    /// Step 1: current admin proposes a successor. Only that account may accept.
    pub fn propose_admin(env: Env, new_admin: Address) -> Result<(), RegistryError> {
        authorize_admin(&env)?;
        env.storage()
            .instance()
            .set(&DataKey::PendingAdmin, &new_admin);
        renew_instance(&env);
        Ok(())
    }

    /// Revoke an unaccepted nomination without changing the active admin.
    /// The current admin must authorize cancellation.
    pub fn cancel_admin_proposal(env: Env) -> Result<(), RegistryError> {
        authorize_admin(&env)?;
        env.storage().instance().remove(&DataKey::PendingAdmin);
        renew_instance(&env);
        Ok(())
    }

    /// Step 2: future admin must explicitly authorize acceptance.
    pub fn accept_admin(env: Env) -> Result<(), RegistryError> {
        let proposed: Address = env
            .storage()
            .instance()
            .get(&DataKey::PendingAdmin)
            .ok_or(RegistryError::NoPendingAdmin)?;
        proposed.require_auth();
        env.storage().instance().set(&DataKey::Admin, &proposed);
        env.storage().instance().remove(&DataKey::PendingAdmin);
        renew_instance(&env);
        Ok(())
    }

    /// Public release notice. Waiting at least five ledgers lets independent
    /// reviewers compare uploaded WASM against the explicitly approved hash.
    /// Only the current admin may stage or replace an upgrade.
    pub fn propose_upgrade(
        env: Env,
        wasm_hash: BytesN<32>,
        delay_ledgers: u32,
    ) -> Result<(), RegistryError> {
        authorize_admin(&env)?;
        if !Self::is_paused(env.clone()) {
            return Err(RegistryError::UpgradeRequiresPause);
        }
        if wasm_hash == BytesN::from_array(&env, &[0u8; 32]) {
            return Err(RegistryError::InvalidUpgradeHash);
        }
        if !(5..=20_000).contains(&delay_ledgers) {
            return Err(RegistryError::InvalidUpgradeDelay);
        }
        let ready_at_ledger = env
            .ledger()
            .sequence()
            .checked_add(delay_ledgers)
            .ok_or(RegistryError::InvalidUpgradeDelay)?;
        let expires_at_ledger = ready_at_ledger
            .checked_add(10_000)
            .ok_or(RegistryError::InvalidUpgradeDelay)?;
        env.storage().instance().set(
            &DataKey::Upgrade,
            &UpgradeProposal {
                wasm_hash: wasm_hash.clone(),
                ready_at_ledger,
                expires_at_ledger,
            },
        );
        UpgradeStaged {
            wasm_hash: wasm_hash.clone(),
            ready_at_ledger,
            expires_at_ledger,
        }.publish(&env);
        renew_instance(&env);
        Ok(())
    }

    /// Inspect the pending proposal without signing or invoking a payment.
    pub fn pending_upgrade(env: Env) -> Option<UpgradeProposal> {
        env.storage().instance().get(&DataKey::Upgrade)
    }

    /// Cancel even during an emergency pause; current admin must authorize.
    pub fn cancel_upgrade(env: Env) -> Result<(), RegistryError> {
        authorize_admin(&env)?;
        let cancelled: Option<UpgradeProposal> = env.storage().instance().get(&DataKey::Upgrade);

        env.storage().instance().remove(&DataKey::Upgrade);
        if let Some(previous) = cancelled {
            UpgradeCancelled { wasm_hash: previous.wasm_hash }.publish(&env);
        }

        renew_instance(&env);
        Ok(())
    }

    /// Execute only the already published hash, within its approved window,
    /// while paused. The replacement WASM must have been uploaded to Testnet.
    /// State remains at the same contract ID; no constructor is re-run.
    pub fn execute_upgrade(env: Env, wasm_hash: BytesN<32>) -> Result<(), RegistryError> {
        authorize_admin(&env)?;
        if !Self::is_paused(env.clone()) {
            return Err(RegistryError::UpgradeRequiresPause);
        }
        let proposal: UpgradeProposal = env
            .storage()
            .instance()
            .get(&DataKey::Upgrade)
            .ok_or(RegistryError::NoPendingUpgrade)?;
        if proposal.wasm_hash != wasm_hash {
            return Err(RegistryError::InvalidUpgradeHash);
        }
        let current = env.ledger().sequence();
        if current < proposal.ready_at_ledger {
            return Err(RegistryError::UpgradeNotReady);
        }
        if current > proposal.expires_at_ledger {
            return Err(RegistryError::UpgradeExpired);
        }
        env.storage().instance().remove(&DataKey::Upgrade);
        renew_instance(&env);
        UpgradeApplied { wasm_hash: wasm_hash.clone() }.publish(&env);
        env.deployer().update_current_contract_wasm(wasm_hash);
        Ok(())
    }
    /// Admin-controlled global stop. Reversing it also requires admin auth.
    pub fn set_paused(env: Env, paused: bool) -> Result<(), RegistryError> {
        authorize_admin(&env)?;
        env.storage().instance().set(&DataKey::Paused, &paused);
        renew_instance(&env);
        Ok(())
    }
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(true)
    }

    /// A registry flag does not prove an issuer, asset, or payout partner exists.
    pub fn set_enabled(env: Env, corridor: String, enabled: bool) -> Result<(), RegistryError> {
        authorize_admin(&env)?;
        if corridor.is_empty() || corridor.len() > 128 {
            return Err(RegistryError::InvalidCorridorId);
        }
        // Do not stage newly enabled flags during an emergency pause.
        // Disabling an entry remains allowed while paused.
        if enabled && Self::is_paused(env.clone()) {
            return Err(RegistryError::Paused);
        }
        let key = DataKey::Corridor(corridor.clone());
        // Revocation clears any previously approved digest. Re-enabling
        // never restores a revoked approval: an admin must review it again.
        if !enabled {
            env.storage()
                .persistent()
                .remove(&DataKey::Approval(corridor.clone()));
        }
        env.storage().persistent().set(&key, &enabled);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND);
        renew_instance(&env);
        Ok(())
    }

    /// An administrator approves public evidence of a specific corridor
    /// configuration for at most 100,000 additional ledger sequences.
    /// Requires an already-enabled corridor and a non-paused registry.
    pub fn approve_config(
        env: Env,
        corridor: String,
        config_digest: BytesN<32>,
        expires_at_ledger: u32,
    ) -> Result<(), RegistryError> {
        authorize_admin(&env)?;
        if corridor.is_empty() || corridor.len() > 128 {
            return Err(RegistryError::InvalidCorridorId);
        }
        if Self::is_paused(env.clone()) {
            return Err(RegistryError::Paused);
        }
        if !Self::is_enabled(env.clone(), corridor.clone()) {
            return Err(RegistryError::NotEnabled);
        }
        // A zero digest cannot identify a reviewed off-chain corridor
        // configuration and must not be usable as public approval evidence.
        if config_digest == BytesN::from_array(&env, &[0u8; 32]) {
            return Err(RegistryError::InvalidConfigDigest);
        }
        let current = env.ledger().sequence();
        let Some(span) = expires_at_ledger.checked_sub(current) else {
            return Err(RegistryError::InvalidExpiration);
        };
        if span == 0 || span > 100_000 {
            return Err(RegistryError::InvalidExpiration);
        }
        let key = DataKey::Approval(corridor);
        env.storage().persistent().set(
            &key,
            &CorridorApproval {
                config_digest,
                expires_at_ledger,
            },
        );
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND);
        renew_instance(&env);
        Ok(())
    }

    /// Strict public read: checks enabled state, exact digest and ledger
    /// expiration. No private data, liquidity or settlement is implied.
    pub fn is_enabled_with_digest(env: Env, corridor: String, expected_digest: BytesN<32>) -> bool {
        if expected_digest == BytesN::from_array(&env, &[0u8; 32])
            || !Self::is_enabled(env.clone(), corridor.clone())
        {
            return false;
        }
        matches!(
            env.storage()
                .persistent()
                .get::<_, CorridorApproval>(&DataKey::Approval(corridor)),
            Some(approval)
                if approval.config_digest == expected_digest
                    && env.ledger().sequence() <= approval.expires_at_ledger
        )
    }
    pub fn is_enabled(env: Env, corridor: String) -> bool {
        if corridor.is_empty() || corridor.len() > 128 {
            return false;
        }
        if Self::is_paused(env.clone()) {
            return false;
        }
        env.storage()
            .persistent()
            .get(&DataKey::Corridor(corridor))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests;
