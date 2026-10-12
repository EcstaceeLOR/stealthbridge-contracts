#!/usr/bin/env python3
"""Fail-closed, offline pre-deployment evidence inventory.

No signature or live RPC verification is performed. Any non-pending evidence,
claims of contract deployment or payment capability require an independently
reviewed verifier upgrade and must fail this source-only release gate.
"""
import json
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
PASSPHRASE="Test SDF Network ; September 2015"
GATES=(
    "source_reproducibility",
    "operator_authorization",
    "testnet_transaction_inclusion",
    "onchain_wasm_and_interface",
    "pause_and_recovery_drill",
    "independent_review",
)
CONTRACTS=("corridor-registry","policy-registry","governance-gate")
REQUIRED={
    "schemaVersion","network","networkPassphrase","releaseStatus",
    "privacyVerified","paymentExecutionEnabled","contracts",
    "requiredEvidence","deploymentTransactionHashes","contractAddresses",
    "operatorStatementSignature","rollbackPauseEvidence"
}

def require(condition,reason):
    if not condition:raise ValueError(reason)

def validate_evidence(record,manifest):
    require(type(record) is dict and set(record)==REQUIRED,
            "missing or extra deployment evidence fields")
    require(type(record["schemaVersion"]) is int and record["schemaVersion"]==1,
            "unsupported evidence schemaVersion")
    require(record["network"]=="testnet" and record["networkPassphrase"]==PASSPHRASE,
            "wrong Stellar network")
    require(record["releaseStatus"]=="not-deployed"
            and record["privacyVerified"] is False
            and record["paymentExecutionEnabled"] is False,
            "unsupported on-chain or financial verification claims")
    require(record["contracts"]==list(CONTRACTS),"incorrect registry inventory or order")
    evidence=record["requiredEvidence"]
    require(type(evidence) is list and len(evidence)==len(GATES),"incomplete evidence gates")
    for position,(row,gate) in enumerate(zip(evidence,GATES)):
        require(type(row) is dict and set(row)=={"gate","status","receiptSha256"}
                and row["gate"]==gate and row["status"]=="pending"
                and row["receiptSha256"] is None,
                f"gate {position} must remain pending until independently verified")
    require(record["deploymentTransactionHashes"]==[] and record["contractAddresses"]=={},
            "unsupported transaction or deployed-contract identity claims")
    require(record["operatorStatementSignature"] is None
            and record["rollbackPauseEvidence"] is None,
            "unverified signatures and rollback evidence cannot be advertised")
    require(type(manifest) is dict and manifest.get("network")=="testnet"
            and manifest.get("status")=="not-deployed" and manifest.get("verified") is False
            and manifest.get("contractAddresses")=={} and manifest.get("txHashes")==[],
            "deployed manifest cannot coexist with pending evidence")
    return len(GATES)

def main():
    evidence=json.loads((ROOT/"deployments/testnet/evidence-chain.v1.json").read_text())
    manifest=json.loads((ROOT/"deployments/testnet/manifest.json").read_text())
    count=validate_evidence(evidence,manifest)
    print(f"PASS: {count} deployment evidence gates remain pending; no testnet deployment attested")

if __name__=="__main__":main()
