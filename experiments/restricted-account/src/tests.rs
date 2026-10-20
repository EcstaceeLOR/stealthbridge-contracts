extern crate std;

use super::*;
use ed25519_dalek::{Signer, SigningKey};
use soroban_sdk::{
    auth::{Context, ContractContext},
    testutils::Address as _,
    vec, IntoVal, Symbol,
};

fn test_account(env: &Env, owner: &SigningKey, allowed: &Address) -> Address {
    env.register(
        RestrictedAccount,
        (
            BytesN::from_array(env, &owner.verifying_key().to_bytes()),
            allowed.clone(),
        ),
    )
}

fn signed_payload(env: &Env, owner: &SigningKey) -> (BytesN<32>, BytesN<64>) {
    let payload = BytesN::<32>::from_array(env, &[5; 32]);
    let signature = owner.sign(&payload.to_array()).to_bytes();
    (payload, BytesN::<64>::from_array(env, &signature))
}

fn allowed_call(env: &Env, contract: &Address, name: &str) -> Context {
    Context::Contract(ContractContext {
        contract: contract.clone(),
        fn_name: Symbol::new(env, name),
        args: vec![env],
    })
}

#[test]
fn ed25519_signature_only_grants_exact_preselected_context() {
    let env = Env::default();
    let signing_key = SigningKey::from_bytes(&[7; 32]);
    let allowed_contract = Address::generate(&env);
    let account = test_account(&env, &signing_key, &allowed_contract);
    let (payload, signature) = signed_payload(&env, &signing_key);
    assert_eq!(
        env.try_invoke_contract_check_auth::<AuthError>(
            &account,
            &payload,
            signature.into_val(&env),
            &vec![&env, allowed_call(&env, &allowed_contract, "authorize")],
        ),
        Ok(())
    );
}

#[test]
fn rejects_replayed_context_for_wrong_method_or_target() {
    let env = Env::default();
    let signing_key = SigningKey::from_bytes(&[8; 32]);
    let allowed_contract = Address::generate(&env);
    let forbidden_contract = Address::generate(&env);
    let account = test_account(&env, &signing_key, &allowed_contract);
    let (payload, signature) = signed_payload(&env, &signing_key);
    for contexts in [
        vec![&env],
        vec![&env, allowed_call(&env, &allowed_contract, "transfer")],
        vec![&env, allowed_call(&env, &forbidden_contract, "authorize")],
        vec![
            &env,
            allowed_call(&env, &allowed_contract, "authorize"),
            allowed_call(&env, &allowed_contract, "authorize"),
        ],
    ] {
        let verdict = env.try_invoke_contract_check_auth::<AuthError>(
            &account,
            &payload,
            signature.clone().into_val(&env),
            &contexts,
        );
        assert!(verdict.is_err());
    }
}

#[test]
fn bad_signature_and_changed_payload_reject() {
    let env = Env::default();
    let owner = SigningKey::from_bytes(&[9; 32]);
    let attacker = SigningKey::from_bytes(&[10; 32]);
    let allowed_contract = Address::generate(&env);
    let account = test_account(&env, &owner, &allowed_contract);
    let (payload, _) = signed_payload(&env, &owner);
    let (_, attacker_signature) = signed_payload(&env, &attacker);
    let contexts = vec![&env, allowed_call(&env, &allowed_contract, "authorize")];
    assert!(env
        .try_invoke_contract_check_auth::<AuthError>(
            &account, &payload, attacker_signature.into_val(&env), &contexts
        )
        .is_err());
    let other_payload = BytesN::<32>::from_array(&env, &[6; 32]);
    let (_, original_signature) = signed_payload(&env, &owner);
    assert!(env
        .try_invoke_contract_check_auth::<AuthError>(
            &account, &other_payload, original_signature.into_val(&env), &contexts
        )
        .is_err());
}

#[test]
fn signed_zero_arg_authorize_must_reject_argument_smuggling() {
    let env = Env::default();
    let owner = SigningKey::from_bytes(&[11; 32]);
    let allowed_contract = Address::generate(&env);
    let account = test_account(&env, &owner, &allowed_contract);
    let (payload, signature) = signed_payload(&env, &owner);
    // A correctly signed payload is not sufficient if the host auth context
    // contains arguments. An unchecked argument could later represent a value,
    // recipient, payment or privileged operation outside the intended scope.
    let with_arguments = Context::Contract(ContractContext {
        contract: allowed_contract.clone(),
        fn_name: Symbol::new(&env, "authorize"),
        args: vec![&env, 7u32.into_val(&env)],
    });
    assert!(env.try_invoke_contract_check_auth::<AuthError>(
        &account,
        &payload,
        signature.into_val(&env),
        &vec![&env, with_arguments],
    ).is_err());
    // Public discovery of this C-address does not establish ownership or
    // turn this source-only research contract into an executable payment rail.
}
