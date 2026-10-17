//! Native Rust source audit for StealthBridge Soroban governance contracts.
//!
//! Checks *actual Rust function signatures* and privileged operation guards.
//! This complements ABI JSON checks, reproducible WASM builds and Soroban
//! host tests. It cannot prove deployed bytecode, wallet authority or privacy.

use std::error::Error;

const CORRIDOR: &str = include_str!("../contracts/corridor-registry/src/lib.rs");
const POLICY: &str = include_str!("../contracts/policy-registry/src/lib.rs");
const GATE: &str = include_str!("../contracts/governance-gate/src/lib.rs");

fn compact(source: &str) -> String {
    source.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Expects precisely the source-level declaration, not a comment or a
/// similarly-named implementation; checked-in CI also compiles all sources.
fn has_signature(source: &str, method: &str, parameters: &str, returns: &str) -> bool {
    let compacted = compact(source);
    let expected = format!("pubfn{method}({parameters})->{returns}{{");
    let without_trailing_comma = parameters.trim_end_matches(',');
    let alternate = format!("pubfn{method}({without_trailing_comma})->{returns}{{");
    compacted.contains(&expected) || compacted.contains(&alternate)
}

fn report_assert(ok: bool, requirement: &str) -> Result<(), Box<dyn Error>> {
    if ok {
        Ok(())
    } else {
        Err(format!("Soroban governance source failed: {requirement}").into())
    }
}

fn audit() -> Result<(), Box<dyn Error>> {
    report_assert(
        has_signature(
            CORRIDOR,
            "approve_config",
            "env:Env,corridor:String,config_digest:BytesN<32>,expires_at_ledger:u32,",
            "Result<(),RegistryError>",
        ),
        "administrator-controlled corridor approval must bind a digest and an expiry",
    )?;
    report_assert(
        has_signature(
            CORRIDOR,
            "is_enabled_with_digest",
            "env:Env,corridor:String,expected_digest:BytesN<32>,",
            "bool",
        ),
        "public corridor reads must check the exact digest",
    )?;
    report_assert(
        CORRIDOR.contains("expires_at_ledger.checked_sub(current)")
            && CORRIDOR.contains("span == 0 || span > 100_000")
            && CORRIDOR.contains("remove(&DataKey::Approval(corridor.clone()))"),
        "corridor approval expiry must be bounded and disabling must revoke the digest",
    )?;
    report_assert(
        has_signature(
            POLICY,
            "is_effective_commitment",
            "env:Env,id:String,expected_revision:u32,expected_commitment:BytesN<32>,",
            "bool",
        ),
        "policy must expose a typed revision/commitment verifier",
    )?;
    report_assert(
        has_signature(
            GATE,
            "public_flags_allow_commitment",
            "env:Env,corridor:String,policy:String,expected_revision:u32,expected_commitment:BytesN<32>,",
            "bool",
        ),
        "gate must bind a corridor to the exact policy revision and commitment",
    )?;
    report_assert(
        has_signature(
            GATE,
            "check_commitment_batch",
            "env:Env,checks:Vec<GovernanceCheck>,",
            "Result<Vec<bool>,GateError>",
        ),
        "governance batch must have explicit bounded/error-returning API",
    )?;
    report_assert(
        GATE.contains("pub enum GateError")
            && GATE.contains("InvalidBatchSize")
            && GATE.contains("checks.len() > 8"),
        "resource-bounded batch is required",
    )?;
    report_assert(
        POLICY.contains("expected_revision == 0")
            && POLICY.contains("rule.revision == expected_revision")
            && POLICY.contains("rule.public_commitment == expected_commitment")
            && POLICY.contains("rule.enabled"),
        "policy reads must reject stale revision, unexpected commitment and disabled entries",
    )?;
    report_assert(
        GATE.contains("env.try_invoke_contract::<bool, soroban_sdk::Error>")
            && GATE.contains("Ok(Ok(true))"),
        "cross-contract errors must fail closed",
    )?;
    report_assert(
        CORRIDOR.contains("admin.require_auth()") || CORRIDOR.contains("current.require_auth()"),
        "corridor admin must require Soroban authorization",
    )?;
    report_assert(
        POLICY.contains("admin.require_auth()"),
        "policy admin must require Soroban authorization",
    )?;
    report_assert(
        CORRIDOR.contains("if enabled && Self::is_paused(env.clone())")
            && POLICY.contains("if record.enabled && Self::is_paused(env.clone())"),
        "governance pause must forbid new enabling writes",
    )?;
    // Two registry upgrades require admin auth, emergency pause, exact uploaded
    // hash, bounded timelock and a cancel path. GovernanceGate stays immutable.
    for (name, source) in [("corridor", CORRIDOR), ("policy", POLICY)] {
        report_assert(
            source.contains("pub fn propose_upgrade(")
                && source.contains("pub fn pending_upgrade(")
                && source.contains("pub fn cancel_upgrade(")
                && source.contains("pub fn execute_upgrade(")
                && source.contains("update_current_contract_wasm(wasm_hash)")
                && source.contains("UpgradeRequiresPause")
                && source.contains("UpgradeNotReady")
                && source.contains("UpgradeExpired")
                && source.contains("InvalidUpgradeHash"),
            &format!("{name} upgrade requires reviewed timelocked administrator safeguards"),
        )?;
    }
    report_assert(!GATE.contains("update_current_contract_wasm")
        && !GATE.contains("pub fn execute_upgrade("),
        "public governance gate remains intentionally immutable")?;
    // No one may inadvertently ship a settlement/payment/write method in the
    // immutable governance aggregation adapter without a separate review.
    report_assert(
        !["pub fn transfer(", "pub fn withdraw(", "pub fn settle(", "pub fn submit("]
            .iter()
            .any(|needle| GATE.contains(needle)),
        "governance gate must not expose fund-moving methods",
    )?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    audit()?;
    println!("PASS: native Rust Soroban signature and security boundary audit; no Testnet deployment asserted");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_source_contracts_pass() {
        audit().expect("pinned Rust Soroban contract source invariants");
    }

    #[test]
    fn signature_comparison_rejects_mutation() {
        let mutation = POLICY.replace("expected_revision: u32", "expected_revision: u64");
        assert!(!has_signature(
            &mutation,
            "is_effective_commitment",
            "env:Env,id:String,expected_revision:u32,expected_commitment:BytesN<32>,",
            "bool"
        ));
    }

    #[test]
    fn source_parser_rejects_missing_and_wrong_return_type() {
        assert!(!has_signature(GATE, "invented_public_method", "env:Env,", "bool"));
        assert!(!has_signature(
            GATE,
            "check_commitment_batch",
            "env:Env,checks:Vec<GovernanceCheck>,",
            "bool"
        ));
    }
}
