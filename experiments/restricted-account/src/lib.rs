#![no_std]
//! Limited Soroban authorization research; NOT a deployed user smart wallet.
//! Permits only one preselected public contract authorize() operation.
//! No financial, deployment, passkey, or recovery interface is provided.
use soroban_sdk::{
    auth::Context, contract, contracterror, contractimpl, contracttype, symbol_short,
    Address, BytesN, Env, Vec,
};

#[contracttype]
#[derive(Clone)]
enum Key {
    Owner,
    AllowedContract,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AuthError {
    Uninitialized = 1,
    UnsupportedContext = 2,
}

#[contract]
pub struct RestrictedAccount;

#[contractimpl]
impl RestrictedAccount {
    pub fn __constructor(env: Env, owner_public_key: BytesN<32>, allowed_contract: Address) {
        env.storage().instance().set(&Key::Owner, &owner_public_key);
        env.storage().instance().set(&Key::AllowedContract, &allowed_contract);
    }

    pub fn owner_key(env: Env) -> Option<BytesN<32>> {
        env.storage().instance().get(&Key::Owner)
    }

    /// Soroban host auth only; never invoke directly from an application.
    #[allow(non_snake_case)]
    pub fn __check_auth(
        env: Env,
        signature_payload: BytesN<32>,
        signature: BytesN<64>,
        auth_context: Vec<Context>,
    ) -> Result<(), AuthError> {
        let owner: BytesN<32> = env
            .storage()
            .instance()
            .get(&Key::Owner)
            .ok_or(AuthError::Uninitialized)?;
        let allowed: Address = env
            .storage()
            .instance()
            .get(&Key::AllowedContract)
            .ok_or(AuthError::Uninitialized)?;
        if auth_context.len() != 1 {
            return Err(AuthError::UnsupportedContext);
        }
        // Reject contract creation, upgrades, nested calls and all methods
        // except a single inert authorization method with empty arguments.
        match auth_context.get(0) {
            Some(Context::Contract(call))
                if call.contract == allowed
                    && call.fn_name == symbol_short!("authorize")
                    && call.args.is_empty() => {}
            _ => return Err(AuthError::UnsupportedContext),
        }
        env.crypto()
            .ed25519_verify(&owner, &signature_payload.into(), &signature);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
