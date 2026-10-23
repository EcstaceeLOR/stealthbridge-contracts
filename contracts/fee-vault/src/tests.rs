use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _, MockAuth, MockAuthInvoke},
    token, Address, BytesN, Env, IntoVal,
};

fn setup() -> (Env, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let operator = Address::generate(&env);
    let issuer = Address::generate(&env);
    let payer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let asset = sac.address();
    token::StellarAssetClient::new(&env, &asset).mint(&payer, &1_000_000);
    let vault = env.register(FeeVault, (operator.clone(), asset.clone()));
    (env, operator, payer, asset, vault)
}

#[test]
fn deposits_accrue_and_claim_only_to_explicit_beneficiary() {
    let (env, operator, payer, asset, vault_id) = setup();
    let vault = FeeVaultClient::new(&env, &vault_id);
    let token = token::Client::new(&env, &asset);
    assert_eq!(vault.get_admin(), operator);
    assert_eq!(vault.get_beneficiary(), operator);
    assert_eq!(vault.get_asset(), asset);
    assert!(vault.is_paused());
    assert!(vault.claims_paused());
    assert_eq!(vault.claimable_fees(), 0);
    assert_eq!(vault.try_deposit_fee(&payer, &100), Err(Ok(FeeVaultError::Paused)));
    vault.set_paused(&false);
    vault.deposit_fee(&payer, &1_000);
    assert_eq!(vault.claimable_fees(), 1_000);
    assert_eq!(vault.vault_balance(), 1_000);
    assert_eq!(token.balance(&payer), 999_000);
    vault.set_claims_paused(&false);
    vault.claim_fees(&375);
    assert_eq!(vault.claimable_fees(), 625);
    assert_eq!(vault.vault_balance(), 625);
    assert_eq!(token.balance(&operator), 375);
    assert_eq!(vault.try_claim_fees(&626), Err(Ok(FeeVaultError::InsufficientBalance)));
}

#[test]
fn unsolicited_token_transfer_does_not_become_claimable_revenue() {
    let (env, _, payer, asset, vault_id) = setup();
    let vault = FeeVaultClient::new(&env, &vault_id);
    let token = token::Client::new(&env, &asset);
    vault.set_claims_paused(&false);
    token.transfer(&payer, &vault_id, &200);
    assert_eq!(vault.vault_balance(), 200);
    assert_eq!(vault.claimable_fees(), 0);
    vault.set_claims_paused(&false);
    assert_eq!(vault.try_claim_fees(&1), Err(Ok(FeeVaultError::InsufficientBalance)));
    vault.set_paused(&false);
    vault.deposit_fee(&payer, &50);
    assert_eq!(vault.vault_balance(), 250);
    assert_eq!(vault.claimable_fees(), 50);
    vault.claim_fees(&50);
    assert_eq!(vault.vault_balance(), 200);
    assert_eq!(vault.claimable_fees(), 0);
}

#[test]
fn invalid_amounts_and_overdrafts_fail_closed() {
    let (env, _, payer, asset, vault_id) = setup();
    let vault = FeeVaultClient::new(&env, &vault_id);
    let token = token::Client::new(&env, &asset);
    vault.set_paused(&false);
    for amount in [0, -1, i128::MIN] {
        assert_eq!(vault.try_deposit_fee(&payer, &amount), Err(Ok(FeeVaultError::InvalidAmount)));
        assert_eq!(vault.try_claim_fees(&amount), Err(Ok(FeeVaultError::InvalidAmount)));
    }
    assert_eq!(token.balance(&payer), 1_000_000);
    assert_eq!(vault.claimable_fees(), 0);
    assert_eq!(vault.try_claim_fees(&1), Err(Ok(FeeVaultError::InsufficientBalance)));
}

#[test]
fn overflow_and_repeated_claims_reject_without_corrupting_accounting() {
    let (env, _, payer, _, vault_id) = setup();
    let vault = FeeVaultClient::new(&env, &vault_id);
    vault.set_paused(&false);
    vault.deposit_fee(&payer, &1);
    vault.set_claims_paused(&false);
    assert_eq!(
        vault.try_deposit_fee(&payer, &i128::MAX),
        Err(Ok(FeeVaultError::ArithmeticOverflow))
    );
    assert_eq!(vault.claimable_fees(), 1);
    vault.claim_fees(&1);
    assert_eq!(
        vault.try_claim_fees(&1),
        Err(Ok(FeeVaultError::InsufficientBalance))
    );
    assert_eq!(vault.claimable_fees(), 0);
    assert_eq!(vault.vault_balance(), 0);
}

#[test]
fn independent_claim_pause_prevents_compromised_claimant_with_valid_auth() {
    let (env, _, payer, _, vault_id) = setup();
    let vault = FeeVaultClient::new(&env, &vault_id);
    vault.set_paused(&false);
    vault.deposit_fee(&payer, &125);
    assert_eq!(
        vault.try_claim_fees(&125),
        Err(Ok(FeeVaultError::ClaimsPaused))
    );
    assert_eq!(vault.claimable_fees(), 125);
    vault.set_claims_paused(&false);
    vault.set_claims_paused(&true);
    assert_eq!(
        vault.try_claim_fees(&125),
        Err(Ok(FeeVaultError::ClaimsPaused))
    );
    assert_eq!(vault.claimable_fees(), 125);
    vault.set_claims_paused(&false);
    vault.claim_fees(&125);
    assert_eq!(vault.claimable_fees(), 0);
}

#[test]
fn emergency_pause_rejects_new_accrual_not_incident_recovery() {
    let (env, operator, payer, asset, vault_id) = setup();
    let vault = FeeVaultClient::new(&env, &vault_id);
    vault.set_paused(&false);
    vault.deposit_fee(&payer, &100);
    vault.set_paused(&true);
    assert!(vault.claims_paused());
    assert_eq!(vault.try_claim_fees(&100), Err(Ok(FeeVaultError::ClaimsPaused)));
    vault.set_claims_paused(&false);
    assert_eq!(vault.try_deposit_fee(&payer, &1), Err(Ok(FeeVaultError::Paused)));
    vault.claim_fees(&100);
    assert_eq!(token::Client::new(&env, &asset).balance(&operator), 100);
    assert_eq!(vault.claimable_fees(), 0);
}

#[test]
fn admin_and_beneficiary_rotate_independently_in_two_steps() {
    let (env, previous, payer, asset, vault_id) = setup();
    let vault = FeeVaultClient::new(&env, &vault_id);
    let new_admin = Address::generate(&env);
    let new_beneficiary = Address::generate(&env);
    vault.set_paused(&false);
    vault.deposit_fee(&payer, &100);
    vault.set_claims_paused(&false);
    vault.propose_admin(&new_admin);
    assert_eq!(vault.pending_admin(), Some(new_admin.clone()));
    vault.accept_admin();
    assert_eq!(vault.get_admin(), new_admin);
    assert_eq!(vault.get_beneficiary(), previous);
    vault.claim_fees(&10);
    assert_eq!(token::Client::new(&env, &asset).balance(&previous), 10);
    vault.propose_beneficiary(&new_beneficiary);
    assert_eq!(vault.pending_beneficiary(), Some(new_beneficiary.clone()));
    vault.accept_beneficiary();
    assert_eq!(vault.pending_beneficiary(), None);
    assert_eq!(vault.get_beneficiary(), new_beneficiary);
    vault.claim_fees(&90);
    assert_eq!(token::Client::new(&env, &asset).balance(&new_beneficiary), 90);
    assert_eq!(vault.try_accept_admin(), Err(Ok(FeeVaultError::NoPendingAdmin)));
    assert_eq!(vault.try_accept_beneficiary(), Err(Ok(FeeVaultError::NoPendingBeneficiary)));
}

#[test]
fn cancelled_handover_preserves_both_active_roles() {
    let (env, old, _, _, vault_id) = setup();
    let vault = FeeVaultClient::new(&env, &vault_id);
    vault.propose_admin(&Address::generate(&env));
    vault.propose_beneficiary(&Address::generate(&env));
    vault.cancel_admin_proposal();
    vault.cancel_beneficiary_proposal();
    assert_eq!(vault.pending_admin(), None);
    assert_eq!(vault.pending_beneficiary(), None);
    assert_eq!(vault.get_admin(), old);
    assert_eq!(vault.get_beneficiary(), old);
}

#[test]
fn unauthorized_claim_rotation_and_upgrade_are_rejected() {
    let (env, _, payer, asset, vault_id) = setup();
    let vault = FeeVaultClient::new(&env, &vault_id);
    vault.set_paused(&false);
    vault.deposit_fee(&payer, &500);
    vault.set_claims_paused(&false);
    env.mock_auths(&[]);
    assert!(vault.try_claim_fees(&100).is_err());
    assert!(vault.try_deposit_fee(&payer, &100).is_err());
    assert!(vault.try_set_paused(&true).is_err());
    assert!(vault.try_set_claims_paused(&true).is_err());
    assert!(vault.try_propose_admin(&payer).is_err());
    assert!(vault.try_propose_beneficiary(&payer).is_err());
    assert!(vault.try_propose_upgrade(&BytesN::from_array(&env, &[1; 32]), &5).is_err());
    assert_eq!(token::Client::new(&env, &asset).balance(&vault_id), 500);
    assert_eq!(vault.claimable_fees(), 500);
}

#[test]
fn attacker_auth_does_not_authorize_claim_from_stored_beneficiary() {
    let (env, _, payer, asset, vault_id) = setup();
    let vault = FeeVaultClient::new(&env, &vault_id);
    let attacker = Address::generate(&env);
    vault.set_paused(&false);
    vault.deposit_fee(&payer, &200);
    vault.set_claims_paused(&false);
    env.mock_auths(&[MockAuth {
        address: &attacker,
        invoke: &MockAuthInvoke {
            contract: &vault_id,
            fn_name: "claim_fees",
            args: (100_i128,).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    assert!(vault.try_claim_fees(&100).is_err());
    assert_eq!(vault.claimable_fees(), 200);
    assert_eq!(token::Client::new(&env, &asset).balance(&vault_id), 200);
}

#[test]
fn beneficiary_nomination_cannot_be_accepted_by_an_attacker() {
    let (env, previous, _, _, vault_id) = setup();
    let vault = FeeVaultClient::new(&env, &vault_id);
    let nominee = Address::generate(&env);
    let attacker = Address::generate(&env);
    vault.propose_beneficiary(&nominee);
    env.mock_auths(&[MockAuth {
        address: &attacker,
        invoke: &MockAuthInvoke {
            contract: &vault_id,
            fn_name: "accept_beneficiary",
            args: ().into_val(&env),
            sub_invokes: &[],
        },
    }]);
    assert!(vault.try_accept_beneficiary().is_err());
    assert_eq!(vault.get_beneficiary(), previous);
    assert_eq!(vault.pending_beneficiary(), Some(nominee));
}

#[test]
fn upgrade_requires_pause_nonzero_hash_delay_and_correct_hash() {
    let (env, _, _, _, vault_id) = setup();
    let vault = FeeVaultClient::new(&env, &vault_id);
    let hash = BytesN::from_array(&env, &[42; 32]);
    let wrong = BytesN::from_array(&env, &[17; 32]);
    let zero = BytesN::from_array(&env, &[0; 32]);
    assert_eq!(vault.try_propose_upgrade(&zero, &5), Err(Ok(FeeVaultError::InvalidUpgradeHash)));
    assert_eq!(vault.try_propose_upgrade(&hash, &4), Err(Ok(FeeVaultError::InvalidUpgradeDelay)));
    vault.set_paused(&false);
    assert_eq!(vault.try_propose_upgrade(&hash, &5), Err(Ok(FeeVaultError::UpgradeRequiresPause)));
    vault.set_paused(&true);
    vault.propose_upgrade(&hash, &5);
    assert_eq!(vault.try_execute_upgrade(&wrong), Err(Ok(FeeVaultError::InvalidUpgradeHash)));
    assert_eq!(vault.try_execute_upgrade(&hash), Err(Ok(FeeVaultError::UpgradeNotReady)));
    vault.cancel_upgrade();
    assert_eq!(vault.pending_upgrade(), None);
    assert_eq!(vault.try_execute_upgrade(&hash), Err(Ok(FeeVaultError::NoPendingUpgrade)));
    // A staged hash has a finite window; a stale proposal can never execute.
    vault.propose_upgrade(&hash, &5);
    env.ledger().with_mut(|info| info.sequence_number += 10_006);
    assert_eq!(vault.try_execute_upgrade(&hash), Err(Ok(FeeVaultError::UpgradeExpired)));
}
