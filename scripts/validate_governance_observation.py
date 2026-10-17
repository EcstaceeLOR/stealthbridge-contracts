#!/usr/bin/env python3
"""Validate an observed three-contract Testnet release against pinned build evidence.

Offline only. A valid result checks schema and local artifact digests, NOT live
chain inclusion, contract state freshness, independent audit or payment ability.
"""
import argparse
import json
from pathlib import Path
from preflight_testnet import valid_operator_address
from validate_manifest import PASSPHRASE, contract_id, hex_hash
from verify_artifacts import verify as verify_artifacts

CONTRACT_FILES = {
    "corridor-registry": ("stealthbridge_corridor_registry.wasm", "corridor-registry.abi.json"),
    "policy-registry": ("stealthbridge_policy_registry.wasm", "policy-registry.abi.json"),
    "governance-gate": ("stealthbridge_governance_gate.wasm", "governance-gate.abi.json"),
}
SHARED_KEYS = {
    "contractId", "wasmSha256", "abiSha256", "wasmUploadTx", "deploymentTx",
    "deploymentLedger", "readbackWasmMatches", "observedAdmin",
}
TOP_KEYS = {
    "schemaVersion", "recordType", "network", "networkPassphrase",
    "sourceCommit", "deployedAt", "deployer", "administrator", "contracts",
    "scope", "verification", "notes",
}
SCOPE_FLAGS = {
    "governanceDeployed": True, "privacyVerified": False,
    "paymentExecutionEnabled": False, "platformFeesEnabled": False,
    "fiatPayoutEnabled": False,
}
VERIFICATION_FLAGS = {
    "sourceWasmBuildReproducible": True,
    "onchainWasmReadbackChecked": True,
    "onchainConstructorStateReadbackChecked": True,
    "deploymentTransactionsConfirmedInTestnetHorizon": True,
    "independentExternalAuditCompleted": False,
}

def require(condition, message):
    if not condition:
        raise ValueError(message)

def validate_observation(record, provenance):
    require(type(record) is dict and set(record) == TOP_KEYS,
            "invalid or incomplete observation envelope")
    require(type(record["schemaVersion"]) is int and record["schemaVersion"] == 2,
            "unsupported observation schema")
    require(record["recordType"] == "soroban-testnet-governance-deployment-observation"
            and record["network"] == "testnet" and record["networkPassphrase"] == PASSPHRASE,
            "wrong network or observation type")
    require(hex_hash(record["sourceCommit"], 40) and
            record["sourceCommit"] == provenance.get("sourceCommit"),
            "source commit mismatch")
    require(record["deployedAt"] == "2026-10-17",
            "unexpected release date")
    require(valid_operator_address(record["administrator"])
            and valid_operator_address(record["deployer"])
            and record["administrator"] == record["deployer"],
            "this observed release must use the approved Testnet operator")
    require(type(record["notes"]) is str and bool(record["notes"]),
            "release notes are required")
    require(record["scope"] == SCOPE_FLAGS,
            "governance release must not claim payments, fees, or privacy")
    require(record["verification"] == VERIFICATION_FLAGS,
            "unreviewed or missing observation checks")
    require(provenance.get("dirty") is False and provenance.get("reproducibleBuilds") == 2
            and provenance.get("sdk") == "27.0.6" and provenance.get("targetProtocol") == 27,
            "invalid reproducible source provenance")
    contracts = record["contracts"]
    require(type(contracts) is dict and set(contracts) == set(CONTRACT_FILES),
            "all three and only the three approved governance contracts are required")
    ids, txs = set(), set()
    for name in CONTRACT_FILES:
        observed = contracts[name]
        fields = SHARED_KEYS | ({"observedPaused"} if name != "governance-gate" else
                                {"observedCorridorRegistry", "observedPolicyRegistry"})
        require(type(observed) is dict and set(observed) == fields,
                "incomplete contract receipt for " + name)
        identifier = observed["contractId"]
        require(contract_id(identifier) and identifier not in ids,
                "invalid or duplicate contract ID for " + name)
        ids.add(identifier)
        require(observed["observedAdmin"] == record["administrator"]
                and observed["readbackWasmMatches"] is True,
                "administrator or WASM observation mismatch for " + name)
        if name != "governance-gate":
            require(observed["observedPaused"] is False,
                    "wrong observed initial registry pause state")
        require(type(observed["deploymentLedger"]) is int
                and observed["deploymentLedger"] > 0,
                "invalid on-chain deployment ledger")
        for field in ("wasmUploadTx", "deploymentTx"):
            tx = observed[field]
            require(hex_hash(tx) and tx not in txs, "invalid or duplicate tx ID")
            txs.add(tx)
        release = provenance["contractArtifacts"][name]
        require(observed["wasmSha256"] == release["wasmSha256"] and
                observed["abiSha256"] == release["abiSha256"],
                "source artifact mismatch for " + name)
    gate = contracts["governance-gate"]
    require(gate["observedCorridorRegistry"] == contracts["corridor-registry"]["contractId"]
            and gate["observedPolicyRegistry"] == contracts["policy-registry"]["contractId"],
            "GovernanceGate references different registry contracts")
    return len(ids)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--record", type=Path, default=Path(
        "deployments/testnet/governance-deployment-evidence.v2.json"))
    parser.add_argument("--artifacts", type=Path, default=Path("artifacts"))
    args = parser.parse_args()
    try:
        require(verify_artifacts(args.artifacts) == 3, "invalid release artifacts")
        record = json.loads(args.record.read_text())
        provenance = json.loads((args.artifacts / "provenance.json").read_text())
        validate_observation(record, provenance)
    except (OSError, ValueError, KeyError, TypeError) as exc:
        parser.exit(1, "Invalid deployment observation: " + str(exc) + "\n")
    print("PASS: 3 governance deployment observations match pinned local build evidence.")
    print("NOTE: This offline validator does not independently re-query Testnet or certify payments.")

if __name__ == "__main__":
    main()
