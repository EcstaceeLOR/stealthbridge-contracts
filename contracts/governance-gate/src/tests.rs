extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, BytesN, String, Vec};
use stealthbridge_corridor_registry::{CorridorRegistry, CorridorRegistryClient};
use stealthbridge_policy_registry::{PolicyRecord, PolicyRegistry, PolicyRegistryClient};

#[test]
fn observes_both_real_registries_and_fails_closed_during_emergency_pause() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let corridor_address = env.register(CorridorRegistry, (admin.clone(),));
    let policy_address = env.register(PolicyRegistry, (admin.clone(),));
    let gate_address = env.register(
        GovernanceGate,
        (
            admin.clone(),
            corridor_address.clone(),
            policy_address.clone(),
        ),
    );
    let gate = GovernanceGateClient::new(&env, &gate_address);
    let corridors = CorridorRegistryClient::new(&env, &corridor_address);
    let policies = PolicyRegistryClient::new(&env, &policy_address);
    let corridor = String::from_str(&env, "testnet_public_corridor");
    let policy = String::from_str(&env, "testnet_public_policy");

    assert_eq!(gate.get_admin(), Some(admin));
    assert_eq!(gate.corridor_registry(), Some(corridor_address));
    assert_eq!(gate.policy_registry(), Some(policy_address));
    assert!(!gate.public_flags_allow(&corridor, &policy));
    corridors.set_enabled(&corridor, &true);
    assert!(!gate.public_flags_allow(&corridor, &policy));
    policies.set_rule(
        &policy,
        &PolicyRecord {
            revision: 1,
            enabled: true,
            public_commitment: BytesN::from_array(&env, &[7u8; 32]),
        },
    );
    assert!(gate.public_flags_allow(&corridor, &policy));
    corridors.set_paused(&true);
    assert!(!gate.public_flags_allow(&corridor, &policy));
    corridors.set_paused(&false);
    assert!(gate.public_flags_allow(&corridor, &policy));
    policies.set_paused(&true);
    assert!(!gate.public_flags_allow(&corridor, &policy));
    policies.set_paused(&false);
    assert!(gate.public_flags_allow(&corridor, &policy));
    corridors.set_enabled(&corridor, &false);
    assert!(!gate.public_flags_allow(&corridor, &policy));
}

#[test]
fn commitment_bound_gate_requires_exact_active_policy_and_corridor() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let corridor_address = env.register(CorridorRegistry, (admin.clone(),));
    let policy_address = env.register(PolicyRegistry, (admin.clone(),));
    let gate_address = env.register(
        GovernanceGate,
        (admin, corridor_address.clone(), policy_address.clone()),
    );
    let gate = GovernanceGateClient::new(&env, &gate_address);
    let corridors = CorridorRegistryClient::new(&env, &corridor_address);
    let policies = PolicyRegistryClient::new(&env, &policy_address);
    let corridor = String::from_str(&env, "operator_vetted_corridor");
    let policy = String::from_str(&env, "public_policy_v1");
    let v1 = BytesN::from_array(&env, &[11_u8; 32]);
    let v2 = BytesN::from_array(&env, &[12_u8; 32]);
    assert!(!gate.public_flags_allow_commitment(&corridor, &policy, &1, &v1));
    corridors.set_enabled(&corridor, &true);
    policies.set_rule(
        &policy,
        &PolicyRecord {
            revision: 1,
            enabled: true,
            public_commitment: v1.clone(),
        },
    );
    assert!(gate.public_flags_allow_commitment(&corridor, &policy, &1, &v1));
    assert!(!gate.public_flags_allow_commitment(
        &corridor,
        &policy,
        &1,
        &BytesN::from_array(&env, &[0u8; 32])
    ));
    assert!(!gate.public_flags_allow_commitment(&corridor, &policy, &1, &v2));
    assert!(!gate.public_flags_allow_commitment(&corridor, &policy, &0, &v1));
    assert!(!gate.public_flags_allow_commitment(&corridor, &policy, &2, &v1));
    policies.set_rule(
        &policy,
        &PolicyRecord {
            revision: 2,
            enabled: true,
            public_commitment: v2.clone(),
        },
    );
    assert!(!gate.public_flags_allow_commitment(&corridor, &policy, &1, &v1));
    assert!(gate.public_flags_allow_commitment(&corridor, &policy, &2, &v2));
    corridors.set_paused(&true);
    assert!(!gate.public_flags_allow_commitment(&corridor, &policy, &2, &v2));
    corridors.set_paused(&false);
    policies.set_paused(&true);
    assert!(!gate.public_flags_allow_commitment(&corridor, &policy, &2, &v2));
    policies.set_paused(&false);
    corridors.set_enabled(&corridor, &false);
    assert!(!gate.public_flags_allow_commitment(&corridor, &policy, &2, &v2));
}
#[test]
fn bounded_batch_returns_ordered_flags_and_rejects_excessive_work() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let corridor_address = env.register(CorridorRegistry, (admin.clone(),));
    let policy_address = env.register(PolicyRegistry, (admin.clone(),));
    let gate_address = env.register(
        GovernanceGate,
        (admin, corridor_address.clone(), policy_address.clone()),
    );
    let gate = GovernanceGateClient::new(&env, &gate_address);
    let corridors = CorridorRegistryClient::new(&env, &corridor_address);
    let policies = PolicyRegistryClient::new(&env, &policy_address);
    let corridor = String::from_str(&env, "batch_corridor");
    let missing = String::from_str(&env, "missing_corridor");
    let policy = String::from_str(&env, "batch_policy");
    let commitment = BytesN::from_array(&env, &[77_u8; 32]);
    corridors.set_enabled(&corridor, &true);
    policies.set_rule(
        &policy,
        &PolicyRecord {
            revision: 9,
            enabled: true,
            public_commitment: commitment.clone(),
        },
    );
    let allowed = GovernanceCheck {
        corridor: corridor.clone(),
        policy: policy.clone(),
        expected_revision: 9,
        expected_commitment: commitment.clone(),
    };
    let unavailable = GovernanceCheck {
        corridor: missing,
        policy,
        expected_revision: 9,
        expected_commitment: commitment,
    };
    let mut checks = Vec::new(&env);
    checks.push_back(allowed.clone());
    checks.push_back(unavailable);
    let results = gate.check_commitment_batch(&checks);
    assert_eq!(results.len(), 2);
    assert_eq!(results.get(0), Some(true));
    assert_eq!(results.get(1), Some(false));
    let empty = Vec::<GovernanceCheck>::new(&env);
    assert_eq!(
        gate.try_check_commitment_batch(&empty),
        Err(Ok(GateError::InvalidBatchSize))
    );
    for _ in 2..9 {
        checks.push_back(allowed.clone());
    }
    assert_eq!(checks.len(), 9);
    assert_eq!(
        gate.try_check_commitment_batch(&checks),
        Err(Ok(GateError::InvalidBatchSize))
    );
    corridors.set_paused(&true);
    let mut single = Vec::new(&env);
    single.push_back(allowed);
    assert_eq!(gate.check_commitment_batch(&single).get(0), Some(false));
}
#[test]
fn invalid_arguments_and_unknown_registry_addresses_never_authorize() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let unknown_corridor = Address::generate(&env);
    let unknown_policy = Address::generate(&env);
    let gate_address = env.register(GovernanceGate, (admin, unknown_corridor, unknown_policy));
    let gate = GovernanceGateClient::new(&env, &gate_address);
    let valid = String::from_str(&env, "public_candidate_only");
    let empty = String::from_str(&env, "");
    let oversized = String::from_str(&env, &"x".repeat(129));
    assert!(!gate.public_flags_allow(&valid, &valid));
    assert!(!gate.public_flags_allow(&empty, &valid));
    assert!(!gate.public_flags_allow(&valid, &empty));
    assert!(!gate.public_flags_allow(&oversized, &valid));
    assert!(!gate.public_flags_allow(&valid, &oversized));
}

#[test]
fn recorded_governance_decision_is_not_reusable_after_policy_revision() {
    let env = Env::default();
    env.mock_all_auths();
    let operator = Address::generate(&env);
    let corridor_addr = env.register(CorridorRegistry, (operator.clone(),));
    let policy_addr = env.register(PolicyRegistry, (operator.clone(),));
    let gate_addr = env.register(
        GovernanceGate,
        (operator, corridor_addr.clone(), policy_addr.clone()),
    );
    let corridors = CorridorRegistryClient::new(&env, &corridor_addr);
    let policies = PolicyRegistryClient::new(&env, &policy_addr);
    let gate = GovernanceGateClient::new(&env, &gate_addr);
    let corridor = String::from_str(&env, "public_reconciliation_corridor");
    let policy = String::from_str(&env, "approval_policy");
    let first = BytesN::from_array(&env, &[21_u8; 32]);
    let revised = BytesN::from_array(&env, &[22_u8; 32]);
    corridors.set_enabled(&corridor, &true);
    policies.set_rule(
        &policy,
        &PolicyRecord {
            revision: 1,
            enabled: true,
            public_commitment: first.clone(),
        },
    );
    let old = GovernanceCheck {
        corridor: corridor.clone(),
        policy: policy.clone(),
        expected_revision: 1,
        expected_commitment: first.clone(),
    };
    let new = GovernanceCheck {
        corridor: corridor.clone(),
        policy: policy.clone(),
        expected_revision: 2,
        expected_commitment: revised.clone(),
    };
    let mut checks = Vec::new(&env);
    checks.push_back(old.clone());
    checks.push_back(new.clone());
    assert_eq!(gate.check_commitment_batch(&checks).get(0), Some(true));
    assert_eq!(gate.check_commitment_batch(&checks).get(1), Some(false));

    policies.set_rule(
        &policy,
        &PolicyRecord {
            revision: 2,
            enabled: true,
            public_commitment: revised,
        },
    );
    // Historical approval is not authority after a policy revision.
    let replaced = gate.check_commitment_batch(&checks);
    assert_eq!(replaced.get(0), Some(false));
    assert_eq!(replaced.get(1), Some(true));
    corridors.set_paused(&true);
    let paused = gate.check_commitment_batch(&checks);
    assert_eq!(paused.get(0), Some(false));
    assert_eq!(paused.get(1), Some(false));
}
