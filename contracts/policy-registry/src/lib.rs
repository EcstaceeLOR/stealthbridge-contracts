#![no_std]
//! Public policy commitments, version control, and an emergency availability
//! switch. No protected amounts, personal information or payout authority.
//! A policy flag is NOT a KYC/compliance decision or a proof verification.
use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, BytesN, Env,
    String,
};
const TTL_THRESHOLD: u32 = 17_280;
const TTL_EXTEND: u32 = 30 * TTL_THRESHOLD;
#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    PendingAdmin,
    Paused,
    Rule(String),
    Upgrade,
}
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyRecord {
    pub revision: u32,
    pub enabled: bool,
    pub public_commitment: BytesN<32>,
}
/// An authorized public upgrade proposal. No secret witnesses or review identities.
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
pub enum PolicyError {
    NotInitialized = 1,
    InvalidRevision = 2,
    StaleRevision = 3,
    InvalidRuleId = 4,
    NoPendingAdmin = 5,
    Paused = 6,
    InvalidCommitment = 7,
    UpgradeRequiresPause = 8,
    InvalidUpgradeDelay = 9,
    InvalidUpgradeHash = 10,
    NoPendingUpgrade = 11,
    UpgradeNotReady = 12,
    UpgradeExpired = 13,
}
#[contract]
pub struct PolicyRegistry;
fn require_admin(env: &Env) -> Result<(), PolicyError> {
    let admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(PolicyError::NotInitialized)?;
    admin.require_auth();
    Ok(())
}
fn extend_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND);
}
#[contractimpl]
impl PolicyRegistry {
    pub fn __constructor(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        extend_instance(&env);
    }
    pub fn admin(env: Env) -> Result<Address, PolicyError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(PolicyError::NotInitialized)
    }

    /// Governance handover is two-phase: current admin nominates and future
    /// admin must authorize acceptance with their own account.
    pub fn pending_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::PendingAdmin)
    }
    pub fn propose_admin(env: Env, successor: Address) -> Result<(), PolicyError> {
        require_admin(&env)?;
        env.storage()
            .instance()
            .set(&DataKey::PendingAdmin, &successor);
        extend_instance(&env);
        Ok(())
    }
    pub fn cancel_admin_proposal(env: Env) -> Result<(), PolicyError> {
        require_admin(&env)?;
        env.storage().instance().remove(&DataKey::PendingAdmin);
        extend_instance(&env);
        Ok(())
    }
    pub fn accept_admin(env: Env) -> Result<(), PolicyError> {
        let successor: Address = env
            .storage()
            .instance()
            .get(&DataKey::PendingAdmin)
            .ok_or(PolicyError::NoPendingAdmin)?;
        successor.require_auth();
        env.storage().instance().set(&DataKey::Admin, &successor);
        env.storage().instance().remove(&DataKey::PendingAdmin);
        extend_instance(&env);
        Ok(())
    }
    /// Publish an already-uploaded WASM hash for an independently reviewed
    /// time-delayed upgrade. Both proposal and execution require the same
    /// current Soroban administrator, and policy updates remain paused.
    pub fn propose_upgrade(
        env: Env,
        wasm_hash: BytesN<32>,
        delay_ledgers: u32,
    ) -> Result<(), PolicyError> {
        require_admin(&env)?;
        if !Self::is_paused(env.clone()) {
            return Err(PolicyError::UpgradeRequiresPause);
        }
        if wasm_hash == BytesN::from_array(&env, &[0; 32]) {
            return Err(PolicyError::InvalidUpgradeHash);
        }
        if !(5..=20_000).contains(&delay_ledgers) {
            return Err(PolicyError::InvalidUpgradeDelay);
        }
        let ready_at_ledger = env
            .ledger()
            .sequence()
            .checked_add(delay_ledgers)
            .ok_or(PolicyError::InvalidUpgradeDelay)?;
        let expires_at_ledger = ready_at_ledger
            .checked_add(10_000)
            .ok_or(PolicyError::InvalidUpgradeDelay)?;
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
        }
        .publish(&env);
        extend_instance(&env);
        Ok(())
    }

    /// Public inspection of the exact pending executable and execution window.
    pub fn pending_upgrade(env: Env) -> Option<UpgradeProposal> {
        env.storage().instance().get(&DataKey::Upgrade)
    }

    /// Explicit administrator cancellation; cancellation requires a signature.
    pub fn cancel_upgrade(env: Env) -> Result<(), PolicyError> {
        require_admin(&env)?;
        let cancelled: Option<UpgradeProposal> = env.storage().instance().get(&DataKey::Upgrade);

        env.storage().instance().remove(&DataKey::Upgrade);
        if let Some(previous) = cancelled {
            UpgradeCancelled {
                wasm_hash: previous.wasm_hash,
            }
            .publish(&env);
        }

        extend_instance(&env);
        Ok(())
    }

    /// Switch WASM at the SAME contract address only after pause, admin auth,
    /// matching hash, timelock and bounded expiry. Never auto-unpause.
    /// Replacement bytecode and its storage schema require independent review.
    pub fn execute_upgrade(env: Env, wasm_hash: BytesN<32>) -> Result<(), PolicyError> {
        require_admin(&env)?;
        if !Self::is_paused(env.clone()) {
            return Err(PolicyError::UpgradeRequiresPause);
        }
        let pending: UpgradeProposal = env
            .storage()
            .instance()
            .get(&DataKey::Upgrade)
            .ok_or(PolicyError::NoPendingUpgrade)?;
        if pending.wasm_hash != wasm_hash {
            return Err(PolicyError::InvalidUpgradeHash);
        }
        let current = env.ledger().sequence();
        if current < pending.ready_at_ledger {
            return Err(PolicyError::UpgradeNotReady);
        }
        if current > pending.expires_at_ledger {
            return Err(PolicyError::UpgradeExpired);
        }
        env.storage().instance().remove(&DataKey::Upgrade);
        extend_instance(&env);
        UpgradeApplied {
            wasm_hash: wasm_hash.clone(),
        }
        .publish(&env);
        env.deployer().update_current_contract_wasm(wasm_hash);
        Ok(())
    }
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(true)
    }
    pub fn set_paused(env: Env, paused: bool) -> Result<(), PolicyError> {
        require_admin(&env)?;
        env.storage().instance().set(&DataKey::Paused, &paused);
        extend_instance(&env);
        Ok(())
    }
    /// \`public_commitment\` is opaque PUBLIC bytes only; no private witness.
    /// Caller must validate the actual policy externally; this stores no oracle.
    pub fn set_rule(env: Env, id: String, record: PolicyRecord) -> Result<(), PolicyError> {
        require_admin(&env)?;
        if id.is_empty() || id.len() > 128 {
            return Err(PolicyError::InvalidRuleId);
        }
        if record.revision == 0 {
            return Err(PolicyError::InvalidRevision);
        }
        // An all-zero digest is a missing public policy commitment. It must
        // never be used to enable a policy; disabling with zero is allowed
        // for emergency revocation without requiring a new config digest.
        if record.enabled && record.public_commitment == BytesN::from_array(&env, &[0u8; 32]) {
            return Err(PolicyError::InvalidCommitment);
        }
        // A pause permits revocation, never staging an enabled policy.
        if record.enabled && Self::is_paused(env.clone()) {
            return Err(PolicyError::Paused);
        }
        let key = DataKey::Rule(id);
        if let Some(current) = env.storage().persistent().get::<_, PolicyRecord>(&key) {
            if record.revision <= current.revision {
                return Err(PolicyError::StaleRevision);
            }
        }
        env.storage().persistent().set(&key, &record);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND);
        extend_instance(&env);
        Ok(())
    }
    pub fn get_rule(env: Env, id: String) -> Option<PolicyRecord> {
        if id.is_empty() || id.len() > 128 {
            return None;
        }
        let key = DataKey::Rule(id);
        env.storage().persistent().get(&key)
    }
    /// Returns false for absent, disabled or globally paused entries.
    /// \`true\` only means operator-governed public config is enabled.
    /// Check the exact public policy revision and commitment.
    /// This is not a ZK-proof, identity, asset or payout authorization.
    pub fn is_effective_commitment(
        env: Env,
        id: String,
        expected_revision: u32,
        expected_commitment: BytesN<32>,
    ) -> bool {
        if expected_revision == 0
            || expected_commitment == BytesN::from_array(&env, &[0u8; 32])
            || Self::is_paused(env.clone())
        {
            return false;
        }
        matches!(
            Self::get_rule(env, id),
            Some(rule)
                if rule.enabled
                    && rule.revision == expected_revision
                    && rule.public_commitment == expected_commitment
        )
    }
    pub fn is_effective(env: Env, id: String) -> bool {
        if Self::is_paused(env.clone()) {
            return false;
        }
        Self::get_rule(env, id)
            .map(|rule| rule.enabled)
            .unwrap_or(false)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{
        Address as _, Events as _, Ledger as _, MockAuth, MockAuthInvoke,
    };
    use soroban_sdk::{Event as _, IntoVal, Val, Vec};
    #[test]
    fn revisions_are_monotonic_and_pause_fails_closed() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let instance = env.register(PolicyRegistry, (admin.clone(),));
        let client = PolicyRegistryClient::new(&env, &instance);
        let name = String::from_str(&env, "policy_test_fixture");
        let commitment = BytesN::from_array(&env, &[7u8; 32]);
        assert_eq!(client.admin(), admin);
        assert!(!client.is_paused());
        assert!(!client.is_effective(&name));
        let v1 = PolicyRecord {
            revision: 1,
            enabled: true,
            public_commitment: commitment.clone(),
        };
        client.set_rule(&name, &v1);
        assert_eq!(client.get_rule(&name), Some(v1));
        assert!(client.is_effective(&name));
        assert!(client
            .try_set_rule(
                &name,
                &PolicyRecord {
                    revision: 1,
                    enabled: false,
                    public_commitment: commitment.clone()
                }
            )
            .is_err());
        assert!(client
            .try_set_rule(
                &name,
                &PolicyRecord {
                    revision: 0,
                    enabled: true,
                    public_commitment: commitment.clone()
                }
            )
            .is_err());
        client.set_paused(&true);
        assert!(!client.is_effective(&name));
        client.set_paused(&false);
        assert!(client.is_effective(&name));
        client.set_rule(
            &name,
            &PolicyRecord {
                revision: 2,
                enabled: false,
                public_commitment: commitment,
            },
        );
        assert!(!client.is_effective(&name));
    }

    #[test]
    fn emergency_pause_refuses_new_enabled_policy_but_allows_revocation() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let contract = env.register(PolicyRegistry, (admin,));
        let client = PolicyRegistryClient::new(&env, &contract);
        let name = String::from_str(&env, "testnet_public_policy");
        let commitment = BytesN::from_array(&env, &[9u8; 32]);
        client.set_rule(
            &name,
            &PolicyRecord {
                revision: 1,
                enabled: true,
                public_commitment: commitment.clone(),
            },
        );
        assert!(client.is_effective(&name));
        client.set_paused(&true);
        let activation = PolicyRecord {
            revision: 2,
            enabled: true,
            public_commitment: commitment.clone(),
        };
        assert_eq!(
            client.try_set_rule(&name, &activation),
            Err(Ok(PolicyError::Paused))
        );
        assert_eq!(client.get_rule(&name).unwrap().revision, 1);
        client.set_rule(
            &name,
            &PolicyRecord {
                revision: 2,
                enabled: false,
                public_commitment: commitment,
            },
        );
        client.set_paused(&false);
        assert!(!client.is_effective(&name));
    }

    #[test]
    fn public_policy_commitment_is_revision_bound_and_fail_closed() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let contract = env.register(PolicyRegistry, (admin,));
        let client = PolicyRegistryClient::new(&env, &contract);
        let name = String::from_str(&env, "testnet_versioned_policy");
        let correct = BytesN::from_array(&env, &[7_u8; 32]);
        let wrong = BytesN::from_array(&env, &[8_u8; 32]);
        assert!(!client.is_effective_commitment(&name, &1, &correct));
        client.set_rule(
            &name,
            &PolicyRecord {
                revision: 1,
                enabled: true,
                public_commitment: correct.clone(),
            },
        );
        assert!(client.is_effective_commitment(&name, &1, &correct));
        assert!(!client.is_effective_commitment(&name, &0, &correct));
        assert!(!client.is_effective_commitment(&name, &2, &correct));
        assert!(!client.is_effective_commitment(&name, &1, &wrong));
        client.set_rule(
            &name,
            &PolicyRecord {
                revision: 2,
                enabled: true,
                public_commitment: wrong.clone(),
            },
        );
        assert!(!client.is_effective_commitment(&name, &1, &correct));
        assert!(client.is_effective_commitment(&name, &2, &wrong));
        client.set_paused(&true);
        assert!(!client.is_effective_commitment(&name, &2, &wrong));
        client.set_rule(
            &name,
            &PolicyRecord {
                revision: 3,
                enabled: false,
                public_commitment: wrong.clone(),
            },
        );
        client.set_paused(&false);
        assert!(!client.is_effective_commitment(&name, &3, &wrong));
    }
    #[test]
    fn administrator_must_be_explicitly_accepted_and_nomination_can_cancel() {
        let env = Env::default();
        env.mock_all_auths();
        let original = Address::generate(&env);
        let successor = Address::generate(&env);
        let contract = env.register(PolicyRegistry, (original.clone(),));
        let client = PolicyRegistryClient::new(&env, &contract);
        assert_eq!(client.pending_admin(), None);
        assert_eq!(
            client.try_accept_admin(),
            Err(Ok(PolicyError::NoPendingAdmin))
        );
        client.propose_admin(&successor);
        assert_eq!(client.pending_admin(), Some(successor.clone()));
        assert_eq!(client.admin(), original);
        client.cancel_admin_proposal();
        assert_eq!(client.pending_admin(), None);
        client.propose_admin(&successor);
        client.accept_admin();
        assert_eq!(client.admin(), successor);
        assert_eq!(client.pending_admin(), None);
        env.mock_auths(&[]);
        assert!(client.try_set_paused(&true).is_err());
        assert!(client.try_cancel_admin_proposal().is_err());
    }
    #[test]
    fn scoped_auth_proves_nominee_must_sign_handover() {
        let env = Env::default();
        let original = Address::generate(&env);
        let successor = Address::generate(&env);
        let outsider = Address::generate(&env);
        let contract = env.register(PolicyRegistry, (original.clone(),));
        let client = PolicyRegistryClient::new(&env, &contract);

        // Unauthenticated or wrong-party nomination never changes state.
        env.mock_auths(&[]);
        assert!(client.try_propose_admin(&successor).is_err());
        env.mock_auths(&[MockAuth {
            address: &outsider,
            invoke: &MockAuthInvoke {
                contract: &contract,
                fn_name: "propose_admin",
                args: (successor.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        assert!(client.try_propose_admin(&successor).is_err());
        assert_eq!(client.pending_admin(), None);
        env.mock_auths(&[MockAuth {
            address: &original,
            invoke: &MockAuthInvoke {
                contract: &contract,
                fn_name: "propose_admin",
                args: (successor.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        client.propose_admin(&successor);
        assert_eq!(client.pending_admin(), Some(successor.clone()));

        // The current administrator may nominate but CANNOT accept for the
        // successor. A stranger may not accept either.
        for signer in [&original, &outsider] {
            env.mock_auths(&[MockAuth {
                address: signer,
                invoke: &MockAuthInvoke {
                    contract: &contract,
                    fn_name: "accept_admin",
                    args: Vec::<Val>::new(&env),
                    sub_invokes: &[],
                },
            }]);
            assert!(client.try_accept_admin().is_err());
            assert_eq!(client.admin(), original);
            assert_eq!(client.pending_admin(), Some(successor.clone()));
        }
        env.mock_auths(&[MockAuth {
            address: &successor,
            invoke: &MockAuthInvoke {
                contract: &contract,
                fn_name: "accept_admin",
                args: Vec::<Val>::new(&env),
                sub_invokes: &[],
            },
        }]);
        client.accept_admin();
        assert_eq!(client.admin(), successor);
        assert_eq!(client.pending_admin(), None);

        // A stale old-admin signature cannot regain control.
        env.mock_auths(&[MockAuth {
            address: &original,
            invoke: &MockAuthInvoke {
                contract: &contract,
                fn_name: "set_paused",
                args: (true,).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        assert!(client.try_set_paused(&true).is_err());
        assert!(!client.is_paused());
    }

    #[test]
    fn invalid_rule_identifiers_fail_closed() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let contract = env.register(PolicyRegistry, (admin,));
        let client = PolicyRegistryClient::new(&env, &contract);
        let record = PolicyRecord {
            revision: 1,
            enabled: true,
            public_commitment: BytesN::from_array(&env, &[4u8; 32]),
        };
        for id in [
            String::from_str(&env, ""),
            String::from_str(&env, &"x".repeat(129)),
        ] {
            assert_eq!(
                client.try_set_rule(&id, &record),
                Err(Ok(PolicyError::InvalidRuleId))
            );
            assert_eq!(client.get_rule(&id), None);
            assert!(!client.is_effective(&id));
        }
    }
    #[test]
    fn unauthorized_mutations_fail() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let instance = env.register(PolicyRegistry, (admin,));
        let client = PolicyRegistryClient::new(&env, &instance);
        env.mock_auths(&[]);
        let id = String::from_str(&env, "fixture");
        let rule = PolicyRecord {
            revision: 1,
            enabled: true,
            public_commitment: BytesN::from_array(&env, &[1u8; 32]),
        };
        assert!(client.try_set_paused(&true).is_err());
        assert!(client.try_set_rule(&id, &rule).is_err());
    }
    /// Deterministic property matrix: revisions never roll back, emergency pause
    /// never authorizes new enabled configurations, and all public reads fail closed.
    #[test]
    fn policy_revision_and_pause_property_matrix() {
        for id_len in [1usize, 32, 64, 128] {
            let env = Env::default();
            env.mock_all_auths();
            let admin = Address::generate(&env);
            let contract = env.register(PolicyRegistry, (admin,));
            let client = PolicyRegistryClient::new(&env, &contract);
            let id = String::from_str(&env, &"p".repeat(id_len));
            let commitment = BytesN::from_array(&env, &[id_len as u8; 32]);
            assert!(!client.is_effective(&id));
            assert!(!client.is_effective_commitment(&id, &1, &commitment));

            for revision in 1u32..=12 {
                let enabled = revision % 2 == 1;
                let rule = PolicyRecord {
                    revision,
                    enabled,
                    public_commitment: commitment.clone(),
                };
                // Pause blocks activation but not revision-monotonic revocation.
                if enabled && revision > 1 {
                    client.set_paused(&true);
                    assert_eq!(
                        client.try_set_rule(&id, &rule),
                        Err(Ok(PolicyError::Paused))
                    );
                    assert!(!client.is_effective(&id));
                    client.set_paused(&false);
                }
                client.set_rule(&id, &rule);
                assert_eq!(client.get_rule(&id), Some(rule.clone()));
                assert_eq!(client.is_effective(&id), enabled);
                assert_eq!(
                    client.is_effective_commitment(&id, &revision, &commitment),
                    enabled
                );
                assert!(!client.is_effective_commitment(&id, &(revision + 1), &commitment));
                assert_eq!(
                    client.try_set_rule(&id, &rule),
                    Err(Ok(PolicyError::StaleRevision))
                );
                assert_eq!(client.get_rule(&id), Some(rule));
            }
        }
    }

    /// Across multiple independent contract instances, arbitrary unauthorized
    /// identities cannot pause or revise public governance state.
    #[test]
    fn unauthorized_policy_writes_are_invariant_under_id_size() {
        for size in [1usize, 8, 64, 128] {
            let env = Env::default();
            let admin = Address::generate(&env);
            let outsider = Address::generate(&env);
            let contract = env.register(PolicyRegistry, (admin.clone(),));
            let client = PolicyRegistryClient::new(&env, &contract);
            let id = String::from_str(&env, &"k".repeat(size));
            let rule = PolicyRecord {
                revision: 1,
                enabled: true,
                public_commitment: BytesN::from_array(&env, &[1u8; 32]),
            };
            env.mock_auths(&[MockAuth {
                address: &outsider,
                invoke: &MockAuthInvoke {
                    contract: &contract,
                    fn_name: "set_rule",
                    args: (id.clone(), rule.clone()).into_val(&env),
                    sub_invokes: &[],
                },
            }]);
            assert!(client.try_set_rule(&id, &rule).is_err());
            assert_eq!(client.get_rule(&id), None);
            assert!(!client.is_effective(&id));
            env.mock_auths(&[MockAuth {
                address: &outsider,
                invoke: &MockAuthInvoke {
                    contract: &contract,
                    fn_name: "set_paused",
                    args: (true,).into_val(&env),
                    sub_invokes: &[],
                },
            }]);
            assert!(client.try_set_paused(&true).is_err());
            assert!(!client.is_paused());
            assert_eq!(client.admin(), admin);
        }
    }
    #[test]
    fn zero_commitment_cannot_activate_policy_but_cannot_block_emergency_revocation() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let address = env.register(PolicyRegistry, (admin,));
        let client = PolicyRegistryClient::new(&env, &address);
        let id = String::from_str(&env, "risk_policy_public");
        let zero = BytesN::from_array(&env, &[0u8; 32]);
        let approved = BytesN::from_array(&env, &[8u8; 32]);
        assert_eq!(
            client.try_set_rule(
                &id,
                &PolicyRecord {
                    revision: 1,
                    enabled: true,
                    public_commitment: zero.clone()
                }
            ),
            Err(Ok(PolicyError::InvalidCommitment))
        );
        assert_eq!(client.get_rule(&id), None);
        client.set_rule(
            &id,
            &PolicyRecord {
                revision: 1,
                enabled: true,
                public_commitment: approved.clone(),
            },
        );
        assert!(client.is_effective_commitment(&id, &1, &approved));
        assert!(!client.is_effective_commitment(&id, &1, &zero));
        assert_eq!(
            client.try_set_rule(
                &id,
                &PolicyRecord {
                    revision: 2,
                    enabled: true,
                    public_commitment: zero.clone()
                }
            ),
            Err(Ok(PolicyError::InvalidCommitment))
        );
        assert!(client.is_effective_commitment(&id, &1, &approved));
        client.set_paused(&true);
        client.set_rule(
            &id,
            &PolicyRecord {
                revision: 2,
                enabled: false,
                public_commitment: zero.clone(),
            },
        );
        client.set_paused(&false);
        assert!(!client.is_effective(&id));
        assert!(!client.is_effective_commitment(&id, &2, &zero));
    }
    #[test]
    fn upgrade_proposal_requires_pause_authorized_admin_and_bounded_nonzero_hash() {
        let env = Env::default();
        env.ledger().with_mut(|l| {
            l.protocol_version = 27;
            l.sequence_number = 100;
            l.max_entry_ttl = TTL_EXTEND + 1000;
        });
        let admin = Address::generate(&env);
        let id = env.register(PolicyRegistry, (admin,));
        let client = PolicyRegistryClient::new(&env, &id);
        let wasm = BytesN::from_array(&env, &[9; 32]);
        assert_eq!(client.pending_upgrade(), None);
        env.mock_auths(&[]);
        assert!(client.try_propose_upgrade(&wasm, &10).is_err());
        env.mock_all_auths();
        assert_eq!(
            client.try_propose_upgrade(&wasm, &10),
            Err(Ok(PolicyError::UpgradeRequiresPause))
        );
        client.set_paused(&true);
        assert_eq!(
            client.try_propose_upgrade(&wasm, &4),
            Err(Ok(PolicyError::InvalidUpgradeDelay))
        );
        assert_eq!(
            client.try_propose_upgrade(&wasm, &20_001),
            Err(Ok(PolicyError::InvalidUpgradeDelay))
        );
        let zero = BytesN::from_array(&env, &[0; 32]);
        assert_eq!(
            client.try_propose_upgrade(&zero, &10),
            Err(Ok(PolicyError::InvalidUpgradeHash))
        );
        client.propose_upgrade(&wasm, &10);
        assert_eq!(
            client.pending_upgrade(),
            Some(UpgradeProposal {
                wasm_hash: wasm.clone(),
                ready_at_ledger: 110,
                expires_at_ledger: 10_110
            })
        );
        env.mock_auths(&[]);
        assert!(client.try_cancel_upgrade().is_err());
        assert_eq!(client.pending_upgrade().unwrap().wasm_hash, wasm);
    }

    #[test]
    fn upgrade_lifecycle_emits_only_public_proposal_and_cancellation_evidence() {
        let env = Env::default();
        env.ledger().with_mut(|l| {
            l.protocol_version = 27;
            l.sequence_number = 100;
            l.max_entry_ttl = TTL_EXTEND + 1000;
        });
        let admin = Address::generate(&env);
        let id = env.register(PolicyRegistry, (admin,));
        let client = PolicyRegistryClient::new(&env, &id);
        env.mock_all_auths();
        client.set_paused(&true);
        let hash = BytesN::from_array(&env, &[9; 32]);
        client.propose_upgrade(&hash, &20);
        assert_eq!(env.events().all(), [
            UpgradeStaged {
                wasm_hash: hash.clone(),
                ready_at_ledger: 120,
                expires_at_ledger: 10_120,
            }.to_xdr(&env, &id)
        ]);
        client.cancel_upgrade();
        assert_eq!(env.events().all(), [
            UpgradeCancelled { wasm_hash: hash }.to_xdr(&env, &id)
        ]);
    }

    #[test]
    fn upgrade_execution_is_hash_bound_time_bound_and_never_bypasses_pause() {
        let env = Env::default();
        env.ledger().with_mut(|l| {
            l.protocol_version = 27;
            l.sequence_number = 100;
            l.max_entry_ttl = TTL_EXTEND + 1000;
        });
        let admin = Address::generate(&env);
        let id = env.register(PolicyRegistry, (admin,));
        let client = PolicyRegistryClient::new(&env, &id);
        let expected = BytesN::from_array(&env, &[9; 32]);
        let wrong = BytesN::from_array(&env, &[7; 32]);
        env.mock_all_auths();
        client.set_paused(&true);
        assert_eq!(
            client.try_execute_upgrade(&expected),
            Err(Ok(PolicyError::NoPendingUpgrade))
        );
        client.propose_upgrade(&expected, &5);
        assert_eq!(
            client.try_execute_upgrade(&wrong),
            Err(Ok(PolicyError::InvalidUpgradeHash))
        );
        assert_eq!(
            client.try_execute_upgrade(&expected),
            Err(Ok(PolicyError::UpgradeNotReady))
        );
        client.set_paused(&false);
        env.ledger().with_mut(|l| l.sequence_number = 106);
        assert_eq!(
            client.try_execute_upgrade(&expected),
            Err(Ok(PolicyError::UpgradeRequiresPause))
        );
        client.set_paused(&true);
        env.ledger().with_mut(|l| l.sequence_number = 10_106);
        assert_eq!(
            client.try_execute_upgrade(&expected),
            Err(Ok(PolicyError::UpgradeExpired))
        );
        assert_eq!(client.pending_upgrade().unwrap().wasm_hash, expected);
        client.cancel_upgrade();
        assert_eq!(client.pending_upgrade(), None);
        client.set_paused(&false);
    }

    #[test]
    fn upgrade_cancel_respects_admin_auth_and_unpause_does_not_clear_proposal() {
        let env = Env::default();
        env.ledger().with_mut(|l| {
            l.protocol_version = 27;
            l.sequence_number = 250;
            l.max_entry_ttl = TTL_EXTEND + 1000;
        });
        let admin = Address::generate(&env);
        let id = env.register(PolicyRegistry, (admin,));
        let client = PolicyRegistryClient::new(&env, &id);
        let hash = BytesN::from_array(&env, &[42; 32]);
        env.mock_all_auths();
        client.set_paused(&true);
        client.propose_upgrade(&hash, &100);
        client.set_paused(&false);
        assert!(client.pending_upgrade().is_some());
        env.mock_auths(&[]);
        assert!(client.try_cancel_upgrade().is_err());
        env.mock_all_auths();
        client.cancel_upgrade();
        assert_eq!(client.pending_upgrade(), None);
    }
}
