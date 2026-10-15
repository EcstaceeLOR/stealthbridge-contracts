extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{
        storage::{Instance as _, Persistent as _},
        Address as _, Events as _, Ledger as _, MockAuth, MockAuthInvoke,
    },
    IntoVal, Val, Vec,
};

struct Fixture {
    env: Env,
    contract: Address,
    admin: Address,
}

impl Fixture {
    fn new() -> Self {
        let env = Env::default();
        env.ledger().with_mut(|l| {
            l.protocol_version = 27;
            l.sequence_number = 100;
            l.min_persistent_entry_ttl = 100;
            l.max_entry_ttl = TTL_EXTEND + 1000;
        });
        let admin = Address::generate(&env);
        // SDK register auto-authorizes constructors; this is setup, not a
        // claim to verify deployment authorization or real signatures.
        let contract = env.register(CorridorRegistry, (admin.clone(),));
        Self {
            env,
            contract,
            admin,
        }
    }

    fn client(&self) -> CorridorRegistryClient<'_> {
        CorridorRegistryClient::new(&self.env, &self.contract)
    }

    fn auth(&self, signer: &Address, function: &str, args: Vec<Val>) {
        self.env.mock_auths(&[MockAuth {
            address: signer,
            invoke: &MockAuthInvoke {
                contract: &self.contract,
                fn_name: function,
                args,
                sub_invokes: &[],
            },
        }]);
    }

    fn write(&self, id: &String, enabled: bool) {
        self.auth(
            &self.admin,
            "set_enabled",
            (id.clone(), enabled).into_val(&self.env),
        );
        self.client().set_enabled(id, &enabled);
    }

    fn ttls(&self, id: &String) -> (u32, u32) {
        self.env.as_contract(&self.contract, || {
            (
                self.env.storage().instance().get_ttl(),
                self.env
                    .storage()
                    .persistent()
                    .get_ttl(&DataKey::Corridor(id.clone())),
            )
        })
    }

    fn advance(&self, ledgers: u32) {
        self.env.ledger().with_mut(|l| l.sequence_number += ledgers);
    }
}

#[test]
fn upgrade_proposals_require_admin_pause_valid_hash_and_bounded_delay() {
    let f = Fixture::new();
    let client = f.client();
    let digest = BytesN::from_array(&f.env, &[9u8; 32]);
    assert_eq!(client.pending_upgrade(), None);
    f.env.mock_auths(&[]);
    assert!(client.try_propose_upgrade(&digest, &10).is_err());
    f.auth(
        &f.admin,
        "propose_upgrade",
        (digest.clone(), 10u32).into_val(&f.env),
    );
    assert_eq!(
        client.try_propose_upgrade(&digest, &10),
        Err(Ok(RegistryError::UpgradeRequiresPause))
    );
    f.auth(&f.admin, "set_paused", (true,).into_val(&f.env));
    client.set_paused(&true);
    for delay in [0, 4, 20_001] {
        f.auth(
            &f.admin,
            "propose_upgrade",
            (digest.clone(), delay).into_val(&f.env),
        );
        assert_eq!(
            client.try_propose_upgrade(&digest, &delay),
            Err(Ok(RegistryError::InvalidUpgradeDelay))
        );
    }
    let zero = BytesN::from_array(&f.env, &[0u8; 32]);
    f.auth(
        &f.admin,
        "propose_upgrade",
        (zero.clone(), 10u32).into_val(&f.env),
    );
    assert_eq!(
        client.try_propose_upgrade(&zero, &10),
        Err(Ok(RegistryError::InvalidUpgradeHash))
    );
    assert_eq!(client.pending_upgrade(), None);
    f.auth(
        &f.admin,
        "propose_upgrade",
        (digest.clone(), 10u32).into_val(&f.env),
    );
    client.propose_upgrade(&digest, &10);
    assert_eq!(
        client.pending_upgrade(),
        Some(UpgradeProposal {
            wasm_hash: digest.clone(),
            ready_at_ledger: 110,
            expires_at_ledger: 10_110,
        })
    );
    f.env.mock_auths(&[]);
    assert!(client.try_cancel_upgrade().is_err());
    assert_eq!(client.pending_upgrade().unwrap().wasm_hash, digest);
    f.auth(&f.admin, "cancel_upgrade", ().into_val(&f.env));
    client.cancel_upgrade();
    assert_eq!(client.pending_upgrade(), None);
}

#[test]
fn upgrade_execution_rejects_wrong_hash_early_execution_expiration_and_unpause() {
    let f = Fixture::new();
    let client = f.client();
    let digest = BytesN::from_array(&f.env, &[9u8; 32]);
    let wrong = BytesN::from_array(&f.env, &[10u8; 32]);
    f.auth(&f.admin, "set_paused", (true,).into_val(&f.env));
    client.set_paused(&true);
    f.auth(
        &f.admin,
        "execute_upgrade",
        (digest.clone(),).into_val(&f.env),
    );
    assert_eq!(
        client.try_execute_upgrade(&digest),
        Err(Ok(RegistryError::NoPendingUpgrade))
    );
    f.auth(
        &f.admin,
        "propose_upgrade",
        (digest.clone(), 5u32).into_val(&f.env),
    );
    client.propose_upgrade(&digest, &5);
    f.auth(
        &f.admin,
        "execute_upgrade",
        (wrong.clone(),).into_val(&f.env),
    );
    assert_eq!(
        client.try_execute_upgrade(&wrong),
        Err(Ok(RegistryError::InvalidUpgradeHash))
    );
    f.auth(
        &f.admin,
        "execute_upgrade",
        (digest.clone(),).into_val(&f.env),
    );
    assert_eq!(
        client.try_execute_upgrade(&digest),
        Err(Ok(RegistryError::UpgradeNotReady))
    );
    f.auth(&f.admin, "set_paused", (false,).into_val(&f.env));
    client.set_paused(&false);
    f.advance(6);
    f.auth(
        &f.admin,
        "execute_upgrade",
        (digest.clone(),).into_val(&f.env),
    );
    assert_eq!(
        client.try_execute_upgrade(&digest),
        Err(Ok(RegistryError::UpgradeRequiresPause))
    );
    f.auth(&f.admin, "set_paused", (true,).into_val(&f.env));
    client.set_paused(&true);
    f.advance(10_000);
    f.auth(
        &f.admin,
        "execute_upgrade",
        (digest.clone(),).into_val(&f.env),
    );
    assert_eq!(
        client.try_execute_upgrade(&digest),
        Err(Ok(RegistryError::UpgradeExpired))
    );
    assert_eq!(client.pending_upgrade().unwrap().wasm_hash, digest);
    f.auth(&f.admin, "cancel_upgrade", ().into_val(&f.env));
    client.cancel_upgrade();
}
#[test]
fn corridor_nomination_cancellation_requires_current_admin() {
    let f = Fixture::new();
    let client = f.client();
    let successor = Address::generate(&f.env);
    f.auth(
        &f.admin,
        "propose_admin",
        (successor.clone(),).into_val(&f.env),
    );
    client.propose_admin(&successor);
    assert_eq!(client.pending_admin(), Some(successor.clone()));
    f.env.mock_auths(&[]);
    assert!(client.try_cancel_admin_proposal().is_err());
    assert_eq!(client.pending_admin(), Some(successor.clone()));
    f.auth(&f.admin, "cancel_admin_proposal", ().into_val(&f.env));
    client.cancel_admin_proposal();
    assert_eq!(client.pending_admin(), None);
    assert_eq!(
        client.try_accept_admin(),
        Err(Ok(RegistryError::NoPendingAdmin))
    );
    assert_eq!(client.get_admin(), f.admin);
}

#[test]
fn missing_pause_state_fails_closed_instead_of_enabling_corridor() {
    let f = Fixture::new();
    let id = String::from_str(&f.env, "fixture_missing_pause_state");
    f.write(&id, true);
    assert!(f.client().is_enabled(&id));
    f.env.as_contract(&f.contract, || {
        f.env.storage().instance().remove(&DataKey::Paused);
    });
    assert!(f.client().is_paused());
    assert!(!f.client().is_enabled(&id));
}

#[test]
fn scoped_authorization_pause_and_handover() {
    let f = Fixture::new();
    let client = f.client();
    let id = String::from_str(&f.env, "fixture_only");
    let next = Address::generate(&f.env);
    assert!(!client.is_enabled(&id));
    f.write(&id, true);
    assert!(client.is_enabled(&id));
    for paused in [true, false] {
        f.auth(&f.admin, "set_paused", (paused,).into_val(&f.env));
        client.set_paused(&paused);
        assert_eq!(client.is_paused(), paused);
        assert_eq!(client.is_enabled(&id), !paused);
    }
    f.auth(&f.admin, "propose_admin", (next.clone(),).into_val(&f.env));
    client.propose_admin(&next);
    assert_eq!(client.get_admin(), f.admin);
    assert_eq!(client.pending_admin(), Some(next.clone()));
    f.auth(&next, "accept_admin", ().into_val(&f.env));
    client.accept_admin();
    assert_eq!(client.get_admin(), next);
    assert_eq!(client.pending_admin(), None);
    f.auth(&next, "set_enabled", (id.clone(), false).into_val(&f.env));
    client.set_enabled(&id, &false);
    assert!(!client.is_enabled(&id));
    assert!(f.env.events().all().events().is_empty());
}

#[test]
fn missing_wrong_and_stale_admin_auth_rejected_without_state_changes() {
    let f = Fixture::new();
    let client = f.client();
    let id = String::from_str(&f.env, "fixture_only");
    let stranger = Address::generate(&f.env);
    f.write(&id, true);
    for signer in [None, Some(&stranger)] {
        f.env.mock_auths(&[]);
        if let Some(signer) = signer {
            f.auth(signer, "set_enabled", (id.clone(), false).into_val(&f.env));
        }
        assert!(client.try_set_enabled(&id, &false).is_err());
        assert!(client.is_enabled(&id));
        if let Some(signer) = signer {
            f.auth(signer, "set_paused", (true,).into_val(&f.env));
        }
        assert!(client.try_set_paused(&true).is_err());
        assert!(!client.is_paused());
        if let Some(signer) = signer {
            f.auth(
                signer,
                "propose_admin",
                (stranger.clone(),).into_val(&f.env),
            );
        }
        assert!(client.try_propose_admin(&stranger).is_err());
        assert_eq!(client.pending_admin(), None);
    }
    // Even the correct signer cannot reuse approval for different arguments.
    f.auth(&f.admin, "set_enabled", (id.clone(), true).into_val(&f.env));
    assert!(client.try_set_enabled(&id, &false).is_err());
    assert!(client.is_enabled(&id));
    f.auth(
        &f.admin,
        "propose_admin",
        (stranger.clone(),).into_val(&f.env),
    );
    client.propose_admin(&stranger);
    f.env.mock_auths(&[]);
    assert!(client.try_accept_admin().is_err());
    f.auth(&f.admin, "accept_admin", ().into_val(&f.env));
    assert!(client.try_accept_admin().is_err());
    assert_eq!(client.get_admin(), f.admin);
    assert_eq!(client.pending_admin(), Some(stranger.clone()));
    f.auth(&stranger, "accept_admin", ().into_val(&f.env));
    client.accept_admin();
    f.auth(
        &f.admin,
        "set_enabled",
        (id.clone(), false).into_val(&f.env),
    );
    assert!(client.try_set_enabled(&id, &false).is_err());
    f.auth(&f.admin, "set_paused", (true,).into_val(&f.env));
    assert!(client.try_set_paused(&true).is_err());
    f.auth(
        &f.admin,
        "propose_admin",
        (f.admin.clone(),).into_val(&f.env),
    );
    assert!(client.try_propose_admin(&f.admin).is_err());
    assert_eq!(
        client.try_accept_admin(),
        Err(Ok(RegistryError::NoPendingAdmin))
    );
}

#[test]
fn invalid_and_paused_corridors_never_appear_active() {
    let f = Fixture::new();
    let client = f.client();
    let empty = String::from_str(&f.env, "");
    let too_long = String::from_str(&f.env, &"x".repeat(129));
    let valid = String::from_str(&f.env, "candidate_not_a_payout");

    assert!(!client.is_enabled(&empty));
    assert!(!client.is_enabled(&too_long));
    assert!(!client.is_enabled(&valid));

    for invalid in [&empty, &too_long] {
        f.auth(
            &f.admin,
            "set_enabled",
            (invalid.clone(), true).into_val(&f.env),
        );
        assert!(client.try_set_enabled(invalid, &true).is_err());
        assert!(!client.is_enabled(invalid));
    }

    f.write(&valid, true);
    assert!(client.is_enabled(&valid));
    f.auth(&f.admin, "set_paused", (true,).into_val(&f.env));
    client.set_paused(&true);
    assert!(!client.is_enabled(&valid));

    // A valid admin cannot stage enabled flags during the emergency pause.
    f.auth(
        &f.admin,
        "set_enabled",
        (valid.clone(), true).into_val(&f.env),
    );
    assert_eq!(
        client.try_set_enabled(&valid, &true),
        Err(Ok(RegistryError::Paused))
    );
    // Disabling entries while paused is still permitted.
    f.write(&valid, false);
    f.auth(&f.admin, "set_paused", (false,).into_val(&f.env));
    client.set_paused(&false);
    assert!(!client.is_enabled(&valid));
}

#[test]
fn digest_bound_corridor_approval_is_expiring_and_revocable() {
    let f = Fixture::new();
    let client = f.client();
    let id = String::from_str(&f.env, "public_vetted_corridor");
    let digest = BytesN::from_array(&f.env, &[3_u8; 32]);
    let forged = BytesN::from_array(&f.env, &[4_u8; 32]);
    assert!(!client.is_enabled_with_digest(&id, &digest));
    f.write(&id, true);
    assert!(!client.is_enabled_with_digest(&id, &digest));
    f.auth(
        &f.admin,
        "approve_config",
        (id.clone(), digest.clone(), 110_u32).into_val(&f.env),
    );
    client.approve_config(&id, &digest, &110);
    assert!(client.is_enabled_with_digest(&id, &digest));
    assert!(!client.is_enabled_with_digest(&id, &forged));
    f.advance(10);
    assert!(client.is_enabled_with_digest(&id, &digest));
    f.advance(1);
    assert!(!client.is_enabled_with_digest(&id, &digest));
    // A newly reviewed expiry does not restore a disabled corridor.
    f.auth(
        &f.admin,
        "approve_config",
        (id.clone(), digest.clone(), 140_u32).into_val(&f.env),
    );
    client.approve_config(&id, &digest, &140);
    f.write(&id, false);
    assert!(!client.is_enabled_with_digest(&id, &digest));
    f.write(&id, true);
    assert!(!client.is_enabled_with_digest(&id, &digest));
}

#[test]
fn corridor_approval_rejects_expired_unbounded_paused_and_unauthorized_writes() {
    let f = Fixture::new();
    let client = f.client();
    let id = String::from_str(&f.env, "approval_bounds");
    let hash = BytesN::from_array(&f.env, &[5_u8; 32]);
    f.auth(
        &f.admin,
        "approve_config",
        (id.clone(), hash.clone(), 105_u32).into_val(&f.env),
    );
    assert_eq!(
        client.try_approve_config(&id, &hash, &105),
        Err(Ok(RegistryError::NotEnabled))
    );
    f.write(&id, true);
    for invalid in [100_u32, 100_101_u32, 0_u32] {
        f.auth(
            &f.admin,
            "approve_config",
            (id.clone(), hash.clone(), invalid).into_val(&f.env),
        );
        assert_eq!(
            client.try_approve_config(&id, &hash, &invalid),
            Err(Ok(RegistryError::InvalidExpiration))
        );
    }
    f.auth(&f.admin, "set_paused", (true,).into_val(&f.env));
    client.set_paused(&true);
    f.auth(
        &f.admin,
        "approve_config",
        (id.clone(), hash.clone(), 110_u32).into_val(&f.env),
    );
    assert_eq!(
        client.try_approve_config(&id, &hash, &110),
        Err(Ok(RegistryError::Paused))
    );
    f.auth(&f.admin, "set_paused", (false,).into_val(&f.env));
    client.set_paused(&false);
    f.env.mock_auths(&[]);
    assert!(client.try_approve_config(&id, &hash, &110).is_err());
    assert!(!client.is_enabled_with_digest(&id, &hash));
}
#[test]
fn ttl_threshold_boundary_and_read_only_behavior() {
    let f = Fixture::new();
    let id = String::from_str(&f.env, "fixture_ttl");
    f.write(&id, true);
    assert_eq!(f.ttls(&id), (TTL_EXTEND, TTL_EXTEND));
    f.advance(TTL_EXTEND - TTL_THRESHOLD - 1);
    f.write(&id, false);
    assert_eq!(f.ttls(&id), (TTL_THRESHOLD + 1, TTL_THRESHOLD + 1));
    f.advance(1);
    assert!(!f.client().is_enabled(&id));
    assert_eq!(f.ttls(&id), (TTL_THRESHOLD, TTL_THRESHOLD));
    f.write(&id, true);
    assert_eq!(f.ttls(&id), (TTL_EXTEND, TTL_EXTEND));
    f.advance(TTL_EXTEND);
    // TTL zero is still live in the current ledger.
    assert!(f.client().is_enabled(&id));
    assert_eq!(f.ttls(&id), (0, 0));
    f.write(&id, false);
    assert_eq!(f.ttls(&id), (TTL_EXTEND, TTL_EXTEND));
}

#[test]
fn pause_renews_instance_but_not_corridor() {
    let f = Fixture::new();
    let id = String::from_str(&f.env, "fixture_ttl");
    f.write(&id, true);
    f.advance(TTL_EXTEND - 1);
    f.auth(&f.admin, "set_paused", (true,).into_val(&f.env));
    f.client().set_paused(&true);
    assert_eq!(f.ttls(&id), (TTL_EXTEND, 1));
    f.advance(1);
    assert!(!f.client().is_enabled(&id));
    // A paused registry cannot renew an enabled entry; disabling it
    // still renews the persistent TTL without changing the pause state.
    f.auth(&f.admin, "set_enabled", (id.clone(), true).into_val(&f.env));
    assert_eq!(
        f.client().try_set_enabled(&id, &true),
        Err(Ok(RegistryError::Paused))
    );
    f.write(&id, false);
    assert_eq!(f.ttls(&id), (TTL_EXTEND - 1, TTL_EXTEND));
}

#[test]
#[ignore = "run via scripts/benchmark.py to produce JSON resource measurements"]
fn resource_profile() {
    for length in [8, 128] {
        let f = Fixture::new();
        let id = String::from_str(&f.env, &"x".repeat(length));
        for (operation, value) in [
            ("set_enabled_new", true),
            ("set_enabled_repeat", true),
            ("set_enabled_change", false),
        ] {
            f.write(&id, value);
            report(&f, operation, length);
        }
        f.client().is_enabled(&id);
        report(&f, "is_enabled", length);
        for paused in [true, false] {
            f.auth(&f.admin, "set_paused", (paused,).into_val(&f.env));
            f.client().set_paused(&paused);
            report(&f, if paused { "pause" } else { "unpause" }, length);
        }
        let next = Address::generate(&f.env);
        f.auth(&f.admin, "propose_admin", (next.clone(),).into_val(&f.env));
        f.client().propose_admin(&next);
        report(&f, "propose_admin", length);
        f.auth(&next, "accept_admin", ().into_val(&f.env));
        f.client().accept_admin();
        report(&f, "accept_admin", length);
        // Contrast rejected unauthenticated write with a scoped mock above.
        f.env.mock_auths(&[]);
        assert!(f.client().try_set_enabled(&id, &true).is_err());
        report(&f, "rejected_no_auth", length);
        f.advance(TTL_EXTEND - TTL_THRESHOLD);
        f.auth(&next, "set_enabled", (id.clone(), true).into_val(&f.env));
        f.client().set_enabled(&id, &true);
        report(&f, "set_enabled_renew_ttl", length);
        let (instance, persistent) = f.ttls(&id);
        assert_eq!((instance, persistent), (TTL_EXTEND, TTL_EXTEND));
        f.advance(TTL_EXTEND + 1);
        assert!(f.client().is_enabled(&id));
        report(&f, "is_enabled_auto_restore_model", length);
    }
}

fn report(f: &Fixture, operation: &str, length: usize) {
    let r = f.env.cost_estimate().resources();
    std::println!(
        "RESOURCE {{\"operation\":\"{}\",\"id_bytes\":{},\"instructions\":{},\"mem_bytes\":{},\"disk_read_entries\":{},\"memory_read_entries\":{},\"write_entries\":{},\"disk_read_bytes\":{},\"write_bytes\":{},\"event_bytes\":{},\"persistent_rent_ledger_bytes\":{},\"persistent_entry_rent_bumps\":{}}}",
        operation, length, r.instructions, r.mem_bytes, r.disk_read_entries, r.memory_read_entries,
        r.write_entries, r.disk_read_bytes, r.write_bytes, r.contract_events_size_bytes,
        r.persistent_rent_ledger_bytes, r.persistent_entry_rent_bumps
    );
}

#[test]
fn expired_persistent_data_is_restored_by_local_host_model() {
    let f = Fixture::new();
    let id = String::from_str(&f.env, "fixture_restore");
    f.write(&id, true);
    f.advance(TTL_EXTEND + 1);
    // The SDK recording host auto-restores archived persistent entries on
    // access. This is not evidence of a submitted restoration transaction.
    assert!(f.client().is_enabled(&id));
    assert_eq!(f.ttls(&id), (99, 99));
    assert_eq!(f.client().get_admin(), f.admin);
    f.write(&id, false);
    assert_eq!(f.ttls(&id), (TTL_EXTEND, TTL_EXTEND));
}

#[test]
fn zero_corridor_config_digest_never_becomes_public_approval() {
    let fixture = Fixture::new();
    fixture.env.mock_all_auths();
    let client = fixture.client();
    let id = String::from_str(&fixture.env, "issuer_review_required");
    let zero = BytesN::from_array(&fixture.env, &[0u8; 32]);
    let reviewed = BytesN::from_array(&fixture.env, &[5u8; 32]);
    client.set_enabled(&id, &true);
    assert_eq!(
        client.try_approve_config(&id, &zero, &150),
        Err(Ok(RegistryError::InvalidConfigDigest))
    );
    assert!(!client.is_enabled_with_digest(&id, &zero));
    client.approve_config(&id, &reviewed, &150);
    assert!(client.is_enabled_with_digest(&id, &reviewed));
    assert!(!client.is_enabled_with_digest(&id, &zero));
    client.set_enabled(&id, &false);
    client.set_enabled(&id, &true);
    assert!(!client.is_enabled_with_digest(&id, &reviewed));
    client.set_paused(&true);
    assert!(!client.is_enabled_with_digest(&id, &reviewed));
}
