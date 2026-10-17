use super::*;
use soroban_sdk::{testutils::Address as _, token, Address, Env};

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
fn paid_fees_are_withdrawable_only_to_current_administrator() {
    let (env, operator, payer, asset, vault_id) = setup();
    let vault = FeeVaultClient::new(&env, &vault_id);
    let token = token::Client::new(&env, &asset);
    assert_eq!(vault.get_admin(), operator);
    assert_eq!(vault.get_asset(), asset);
    assert!(vault.is_paused());
    assert_eq!(vault.vault_balance(), 0);
    assert_eq!(
        vault.try_deposit_fee(&payer, &100),
        Err(Ok(FeeVaultError::Paused))
    );
    vault.set_paused(&false);
    vault.deposit_fee(&payer, &1_000);
    assert_eq!(vault.vault_balance(), 1_000);
    assert_eq!(token.balance(&payer), 999_000);
    assert_eq!(token.balance(&operator), 0);
    vault.claim_fees(&375);
    assert_eq!(vault.vault_balance(), 625);
    assert_eq!(token.balance(&operator), 375);
    assert_eq!(
        vault.try_claim_fees(&626),
        Err(Ok(FeeVaultError::InsufficientBalance))
    );
    assert_eq!(vault.vault_balance(), 625);
}

#[test]
fn invalid_amounts_and_overdrafts_fail_closed() {
    let (env, _, payer, asset, vault_id) = setup();
    let vault = FeeVaultClient::new(&env, &vault_id);
    let token = token::Client::new(&env, &asset);
    vault.set_paused(&false);
    for amount in [0, -1, i128::MIN] {
        assert_eq!(
            vault.try_deposit_fee(&payer, &amount),
            Err(Ok(FeeVaultError::InvalidAmount))
        );
        assert_eq!(
            vault.try_claim_fees(&amount),
            Err(Ok(FeeVaultError::InvalidAmount))
        );
    }
    assert_eq!(token.balance(&payer), 1_000_000);
    assert_eq!(vault.vault_balance(), 0);
    assert_eq!(
        vault.try_claim_fees(&1),
        Err(Ok(FeeVaultError::InsufficientBalance))
    );
}

#[test]
fn emergency_pause_blocks_new_fees_but_not_operator_recovery() {
    let (env, operator, payer, asset, vault_id) = setup();
    let vault = FeeVaultClient::new(&env, &vault_id);
    vault.set_paused(&false);
    vault.deposit_fee(&payer, &100);
    vault.set_paused(&true);
    assert_eq!(
        vault.try_deposit_fee(&payer, &1),
        Err(Ok(FeeVaultError::Paused))
    );
    vault.claim_fees(&100);
    assert_eq!(token::Client::new(&env, &asset).balance(&operator), 100);
    assert_eq!(vault.vault_balance(), 0);
}

#[test]
fn accepted_administrator_handover_changes_claim_destination() {
    let (env, previous, payer, asset, vault_id) = setup();
    let vault = FeeVaultClient::new(&env, &vault_id);
    let successor = Address::generate(&env);
    vault.set_paused(&false);
    vault.deposit_fee(&payer, &100);
    vault.propose_admin(&successor);
    assert_eq!(vault.get_admin(), previous);
    assert_eq!(vault.pending_admin(), Some(successor.clone()));
    vault.claim_fees(&10);
    assert_eq!(token::Client::new(&env, &asset).balance(&previous), 10);
    vault.accept_admin();
    assert_eq!(vault.pending_admin(), None);
    assert_eq!(vault.get_admin(), successor);
    vault.claim_fees(&90);
    assert_eq!(token::Client::new(&env, &asset).balance(&successor), 90);
    assert_eq!(
        vault.try_accept_admin(),
        Err(Ok(FeeVaultError::NoPendingAdmin))
    );
}

#[test]
fn claims_and_deposits_reject_missing_signatures() {
    let (env, _, payer, asset, vault_id) = setup();
    let vault = FeeVaultClient::new(&env, &vault_id);
    vault.set_paused(&false);
    vault.deposit_fee(&payer, &500);
    env.mock_auths(&[]);
    assert!(vault.try_claim_fees(&100).is_err());
    assert!(vault.try_deposit_fee(&payer, &100).is_err());
    assert_eq!(token::Client::new(&env, &asset).balance(&vault_id), 500);
    assert!(vault.try_set_paused(&true).is_err());
}
