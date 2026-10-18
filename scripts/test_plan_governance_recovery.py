"""No signing, network, funding or live wallet access in recovery planner."""
import json
import unittest
from copy import deepcopy
from plan_governance_recovery import plan, RELEASE_ADMIN, CONTRACTS

NEW="GBBCOFZZ3RDQHTC662EVM3UO5SAXQEZB6DL64HLEJYEYJ4KBVL3CDLTE"

def fixture():
    return {
        "schema_version": 1, "network": "testnet",
        "source": "stellar-rpc/getLedgerEntries/contract-instance-storage",
        "observed_ledger_sequence": 5300000,
        "reviewed_release_executable_hashes_confirmed": True,
        "administrator_entries_read_from_current_storage": True,
        "same_administrator_for_all_contracts": True,
        "all_administrators_match_deployment": True,
        "release_administrator": RELEASE_ADMIN,
        "signer_possession_verified": False, "fee_collection_enabled": False,
        "payment_execution_enabled": False, "privacy_verified": False,
        "contracts": [
            {"name": name, "contract_id": address,
             "current_administrator": RELEASE_ADMIN, "pending_administrator": None,
             "paused": None if i == 2 else False,
             "matches_deployment_administrator": True}
            for i, (name, address, _) in enumerate(CONTRACTS)
        ]
    }

class RecoveryPreflightTests(unittest.TestCase):
    def test_produces_only_offline_two_phase_commands(self):
        outcome = plan(fixture(), NEW)
        self.assertEqual(outcome["network"], "testnet")
        self.assertFalse(outcome["safe_to_execute_automatically"])
        self.assertFalse(outcome["financial_execution_enabled"])
        self.assertEqual([c["phase"] for c in outcome["commands_for_review_only"]],
                         ["propose", "accept", "propose", "accept"])
        self.assertNotIn("governance-gate", [c["contract"] for c in outcome["commands_for_review_only"]])
        self.assertIn("--new_admin", outcome["commands_for_review_only"][0]["command"])
        self.assertIn("--successor", outcome["commands_for_review_only"][2]["command"])
        self.assertNotIn("secret", json.dumps(outcome).lower())

    def test_existing_same_nomination_only_needs_reviewed_acceptance(self):
        case = fixture()
        case["contracts"][0]["pending_administrator"] = NEW
        outcome = plan(case, NEW)
        self.assertEqual([c["phase"] for c in outcome["commands_for_review_only"]],
                         ["accept", "propose", "accept"])

    def test_wrong_admin_code_or_mainnet_is_rejected(self):
        for key, val in [
            ("network", "public"), ("fee_collection_enabled", True),
            ("signer_possession_verified", True),
            ("reviewed_release_executable_hashes_confirmed", False)
        ]:
            case=fixture()
            case[key]=val
            with self.subTest(key=key),self.assertRaises(ValueError):
                plan(case, NEW)
        for field, val in [
            ("current_administrator", NEW), ("contract_id", CONTRACTS[1][1]),
            ("pending_administrator", RELEASE_ADMIN)
        ]:
            case=fixture()
            case["contracts"][0][field]=val
            with self.subTest(field=field),self.assertRaises(ValueError):
                plan(case, NEW)

    def test_invalid_successor_and_unchanged_identity_rejected(self):
        for addr in ["G"*56, "S"*56, RELEASE_ADMIN, "", "G" + "A"*55]:
            with self.subTest(addr=addr),self.assertRaises(ValueError):
                plan(fixture(), addr)

    def test_gate_immutable_and_staged_other_party_requires_manual_review(self):
        case=fixture()
        case["contracts"][2]["pending_administrator"]=NEW
        with self.assertRaises(ValueError):plan(case,NEW)
        case=fixture()
        case["contracts"][1]["pending_administrator"]="GAAACAQDAQCQMBYIBEFAWDANBYHRAEISCMKBKFQXDAMRUGY4DUPB7JZX"
        with self.assertRaises(ValueError):plan(case,NEW)

if __name__ == "__main__":
    unittest.main()
