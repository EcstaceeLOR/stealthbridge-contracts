//! Source-only product safety gates. This audit is not a proof verifier,
//! transaction authorization, privacy guarantee or deployment attestation.
const CORRIDOR: &str = include_str!("../contracts/corridor-registry/src/lib.rs");
const POLICY: &str = include_str!("../contracts/policy-registry/src/lib.rs");
const GATE: &str = include_str!("../contracts/governance-gate/src/lib.rs");
const MANIFEST: &str = include_str!("../deployments/testnet/manifest.json");

fn no_financial_writes(source: &str) -> bool {
    const UNSUPPORTED: [&str; 8] = [
        "pub fn transfer(",
        "pub fn withdraw(",
        "pub fn deposit(",
        "pub fn payout(",
        "pub fn submit_payment(",
        "pub fn settle(",
        "pub fn mint(",
        "pub fn burn(",
    ];
    !UNSUPPORTED.iter().any(|method| source.contains(method))
}

fn source_boundary(corridor: &str, policy: &str, gate: &str, manifest: &str) -> Result<(), &'static str> {
    if ![corridor, policy, gate].into_iter().all(no_financial_writes) {
        return Err("Governance source must not become a fund-moving contract");
    }
    if !gate.contains("pub fn public_flags_allow_commitment(")
        || !gate.contains("pub fn check_commitment_batch(")
        || !gate.contains("checks.len() > 8 {")
        || !gate.contains("Ok(Ok(true))")
    {
        return Err("Public governance reads must be bounded and fail closed");
    }
    if !manifest.contains(r#""network": "testnet""#)
        || !manifest.contains(r#""status": "not-deployed""#)
        || !manifest.contains(r#""verified": false"#)
        || !manifest.contains(r#""contractAddresses": {}"#)
        || !manifest.contains(r#""txHashes": []"#)
    {
        return Err("Undeployed Testnet release boundary changed; manual attestation required");
    }
    Ok(())
}

fn main() -> Result<(), &'static str> {
    source_boundary(CORRIDOR, POLICY, GATE, MANIFEST)?;
    println!("PASS: governance remains read-only and Testnet deployment unverified");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reviewed_sources_are_non_financial() {
        source_boundary(CORRIDOR, POLICY, GATE, MANIFEST).unwrap();
    }

    #[test]
    fn a_new_fund_moving_method_requires_review() {
        let mutated = format!("{GATE}\npub fn transfer(");
        assert!(source_boundary(CORRIDOR, POLICY, &mutated, MANIFEST).is_err());
    }

    #[test]
    fn claimed_deployment_is_rejected() {
        let claimed = MANIFEST.replace("not-deployed", "deployed");
        assert!(source_boundary(CORRIDOR, POLICY, GATE, &claimed).is_err());
    }

    #[test]
    fn missing_batch_guard_is_rejected() {
        let mutated = GATE.replace("checks.len() > 8", "checks.len() > 8000");
        assert!(source_boundary(CORRIDOR, POLICY, &mutated, MANIFEST).is_err());
    }
}
