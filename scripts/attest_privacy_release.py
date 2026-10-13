#!/usr/bin/env python3
"""Single source-of-truth pre-execution privacy decision for both *distinct* rails.

FAIL CLOSED. Green CI means evidence rules enforce NO_GO, *not* ZK proof success.
No wallet, network RPC, payment, signing, witness, private data or secret input.
"""
import json
from pathlib import Path

from check_privacy_evidence import validate as validate_vectors
from check_issuer_capabilities import validate as validate_issuer
from verify_upstream_source_lock import validate_manifest as validate_source_lock

ROOT = Path(__file__).resolve().parents[1]
REQUIRED_GATES = (
    "cryptographicSoundness", "independentAudit", "crossWalletReproduction",
    "publicDisclosureReview", "ledgerEvidence", "issuerOrPoolAuthorization",
)
EXPECTED_RAILS = ("confidential-token", "private-payments")
EXPECTED_INTERFACE = {"corridor-registry", "policy-registry", "governance-gate"}


class UnsafePrivacyRelease(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise UnsafePrivacyRelease(message)


def load(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def assess(manifest, interface, vectors, issuer, source_lock):
    """Validate source-only inputs; reject attempts to silently promote flags.

    A future, separately audited verifier must introduce a NEW versioned
    protocol that accepts real cryptographic evidence. This v1 guard is
    intentionally incapable of switching a rail to payment-ready.
    """
    try:
        validate_vectors(vectors)
        validate_issuer(issuer)
        validate_source_lock(source_lock)
    except (ValueError, AssertionError, KeyError, TypeError) as exc:
        raise UnsafePrivacyRelease("required independent privacy release input rejected") from exc

    require(type(manifest) is dict and
            set(manifest) == {
                "schemaVersion", "network", "status", "verified",
                "contractAddresses", "assetIssuers", "txHashes", "notes",
            } and
            type(manifest["schemaVersion"]) is int and manifest["schemaVersion"] == 1 and
            manifest["network"] == "testnet" and
            manifest["status"] == "not-deployed" and
            manifest["verified"] is False and
            manifest["contractAddresses"] == {} and
            manifest["assetIssuers"] == {} and manifest["txHashes"] == [],
            "on-chain deployment cannot be inferred from source or CI")
    require(type(interface) is dict and
            set(interface) == {
                "schemaVersion", "network", "status", "disclaimer", "contracts",
            } and
            interface["schemaVersion"] == 1 and
            interface["network"] == "testnet" and
            interface["status"] == "source-interface-only" and
            isinstance(interface["disclaimer"], str) and len(interface["disclaimer"]) > 35 and
            type(interface["contracts"]) is dict and
            set(interface["contracts"]) == EXPECTED_INTERFACE,
            "public Soroban interfaces cannot imply deployed privacy or payments")
    for name, entry in interface["contracts"].items():
        require(type(entry) is dict and set(entry) == {"source", "reads", "writes"}
                and type(entry["source"]) is str
                and entry["source"].startswith("contracts/")
                and type(entry["reads"]) is dict
                and type(entry["writes"]) is list,
                "unreviewed Soroban source interface " + name)
    require(type(vectors["requiredGates"]) is dict and
            set(vectors["requiredGates"]) == set(REQUIRED_GATES),
            "required independent evidence dimensions missing")
    require(tuple(rail["rail"] for rail in vectors["rails"]) == EXPECTED_RAILS,
            "distinct proof systems must not be conflated")
    require(vectors["rails"][0]["proofSystem"] == "UltraHonk" and
            vectors["rails"][1]["proofSystem"] == "Groth16",
            "two different proof systems are required")
    require(vectors["rails"][0]["sourceProtocol"] == 29 and
            issuer["local"]["targetProtocol"] == 27,
            "toolchain/protocol mismatch requires separate signed review")
    require(issuer["local"]["paymentExecutionEnabled"] is False and
            vectors["paymentExecutionEnabled"] is False and
            source_lock["paymentExecutionEnabled"] is False,
            "payment execution requires a new audited evidence protocol")
    require(issuer["decision"] == "NO_GO" and vectors["conclusion"] == "NO_GO",
            "source-only privacy review cannot be marked GO")
    assert_forbidden = ("verified", "deployed", "proof-passed")
    for entry in vectors["rails"]:
        for case in entry["vectors"]:
            require(case["evidence"] is None, "unsupported cryptographic receipt or wallet witness")
        require(entry["rail"] not in assert_forbidden,
                "no verified rail may be inferred from source tests")

    blockers = [
        {"gate": key, "status": "missing-independent-evidence"}
        for key in REQUIRED_GATES
    ]
    require(all(v is False for v in vectors["requiredGates"].values()),
            "no independent gate has been established")
    return {
        "schemaVersion": 1,
        "network": "testnet",
        "decision": "NO_GO",
        "paymentExecutionEnabled": False,
        "contractDeploymentVerified": False,
        "upstreamSourceIntegrityOnly": True,
        "rails": {
            "confidential-token": {"proofSystem": "UltraHonk", "verified": False},
            "private-payments": {"proofSystem": "Groth16", "verified": False},
        },
        "blockers": blockers,
        "limitations": "Source/test-vector/CI validation does not prove private amounts, deployed pool, audit, issuer, wallet interoperability or payouts.",
    }


def main():
    report = assess(
        load("deployments/testnet/manifest.json"),
        load("integrations/public-soroban-interface.v1.json"),
        load("integrations/privacy-evidence-vectors.v1.json"),
        load("integrations/confidential-tokens/capability-review.v1.json"),
        load("integrations/upstream-source-lock.v1.json"),
    )
    print(json.dumps(report, indent=2, sort_keys=True))
    print("PASS: NO_GO pre-execution gates; independent Testnet privacy verification remains REQUIRED")


if __name__ == "__main__":
    main()
