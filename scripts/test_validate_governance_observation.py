"""The Testnet observation parser rejects malformed, unsupported or overclaimed records.

These tests are offline: transaction inclusion and bytecode reads require a live
external RPC and cannot be certified merely by checking static JSON.
"""
from copy import deepcopy
import json
from pathlib import Path
import unittest

from validate_governance_observation import validate_observation

ROOT = Path(__file__).resolve().parents[1]
RECORD = ROOT / "deployments/testnet/governance-deployment-evidence.v2.json"


class GovernanceObservationTests(unittest.TestCase):
    def setUp(self):
        self.record = json.loads(RECORD.read_text())
        self.provenance = {
            "sourceCommit": self.record["sourceCommit"],
            "dirty": False, "reproducibleBuilds": 2,
            "sdk": "27.0.6", "targetProtocol": 27,
            "contractArtifacts": {
                name: {
                    "wasmSha256": contract["wasmSha256"],
                    "abiSha256": contract["abiSha256"],
                } for name, contract in self.record["contracts"].items()
            },
        }

    def check(self, record):
        validate_observation(record, self.provenance)

    def test_expected_local_release_observation(self):
        self.assertEqual(validate_observation(self.record, self.provenance), 3)

    def test_missing_registry_is_rejected(self):
        bad = deepcopy(self.record)
        del bad["contracts"]["policy-registry"]
        with self.assertRaises(ValueError): self.check(bad)

    def test_fake_contractor_address_is_rejected(self):
        bad = deepcopy(self.record)
        bad["contracts"]["corridor-registry"]["contractId"] = "C" * 56
        with self.assertRaises(ValueError): self.check(bad)

    def test_swapped_gate_bindings_are_rejected(self):
        bad = deepcopy(self.record)
        bad["contracts"]["governance-gate"]["observedCorridorRegistry"] = bad["contracts"]["policy-registry"]["contractId"]
        with self.assertRaises(ValueError): self.check(bad)

    def test_payment_fee_and_privacy_overclaims_are_rejected(self):
        for flag in ("privacyVerified", "platformFeesEnabled", "paymentExecutionEnabled"):
            with self.subTest(flag=flag):
                bad = deepcopy(self.record)
                bad["scope"][flag] = True
                with self.assertRaises(ValueError): self.check(bad)

    def test_wrong_provenance_or_tampered_wasm_is_rejected(self):
        bad = deepcopy(self.record)
        bad["contracts"]["governance-gate"]["wasmSha256"] = "f" * 64
        with self.assertRaises(ValueError): self.check(bad)
        other = deepcopy(self.provenance)
        other["sourceCommit"] = "d" * 40
        with self.assertRaises(ValueError): validate_observation(self.record, other)

    def test_duplicate_contract_or_receipt_is_rejected(self):
        for field in ("contractId", "deploymentTx"):
            with self.subTest(field=field):
                bad = deepcopy(self.record)
                bad["contracts"]["policy-registry"][field] = bad["contracts"]["corridor-registry"][field]
                with self.assertRaises(ValueError): self.check(bad)

    def test_unreviewed_mutations_are_rejected(self):
        for field, value in (("network", "mainnet"), ("schemaVersion", 3), ("extra", "unsanctioned")):
            with self.subTest(field=field):
                bad = deepcopy(self.record)
                bad[field] = value
                with self.assertRaises(ValueError): self.check(bad)


if __name__ == "__main__":
    unittest.main()
