"""No signing, network, funding or live wallet access in recovery planner."""
import json
import re
import unittest
from time import time
from copy import deepcopy
from plan_governance_recovery import plan, RELEASE_ADMIN, CONTRACTS

NEW="GBBCOFZZ3RDQHTC662EVM3UO5SAXQEZB6DL64HLEJYEYJ4KBVL3CDLTE"

def fixture():
    return {
        "schema_version": 1, "network": "testnet",
        "source": "stellar-rpc/getLedgerEntries/contract-instance-storage",
        "observed_ledger_sequence": 5300000,
        "observed_ledger_closed_at_unix": str(int(time())),
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
        for action in outcome["commands_for_review_only"]:
            self.assertFalse(any(re.fullmatch(r"S[A-Z2-7]{55}", str(arg)) for arg in action["command"]))

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

    def test_replayed_stale_future_and_missing_administrator_observations_fail(self):
        now = int(time())
        for timestamp in [str(now-181), str(now+31), "0",
                          "2031-01-01", "", "999999999999999999999"]:
            case=fixture()
            case["observed_ledger_closed_at_unix"]=timestamp
            with self.subTest(timestamp=timestamp),self.assertRaises(ValueError):
                plan(case, NEW)
        case=fixture()
        del case["observed_ledger_closed_at_unix"]
        with self.assertRaises(ValueError):
            plan(case, NEW)

    def test_invalid_successor_and_unchanged_identity_rejected(self):
        for addr in ["G"*56, "S"*56, RELEASE_ADMIN, "", "G" + "A"*55]:
            with self.subTest(addr=addr),self.assertRaises(ValueError):
                plan(fixture(), addr)

    def test_resume_after_first_registry_is_already_transferred(self):
        case=fixture()
        case["contracts"][0]["current_administrator"]=NEW
        case["contracts"][0]["matches_deployment_administrator"]=False
        case["same_administrator_for_all_contracts"]=False
        case["all_administrators_match_deployment"]=False
        result=plan(case,NEW)
        self.assertEqual(result["migration_state"],"in-progress")
        self.assertEqual(result["registries_already_migrated"],["corridor-registry"])
        self.assertEqual(result["registries_remaining"],["policy-registry"])
        self.assertEqual([c["phase"] for c in result["commands_for_review_only"]],
                         ["propose","accept"])
        self.assertEqual(result["commands_for_review_only"][0]["contract"],"policy-registry")
        self.assertFalse(result["signed"])
        self.assertFalse(result["broadcast"])

    def test_fully_migrated_registries_have_no_more_recovery_commands(self):
        case=fixture()
        for i in [0,1]:
            case["contracts"][i]["current_administrator"]=NEW
            case["contracts"][i]["matches_deployment_administrator"]=False
        # Immutable GovernanceGate still records the original administrator.
        case["same_administrator_for_all_contracts"]=False
        case["all_administrators_match_deployment"]=False
        result=plan(case,NEW)
        self.assertEqual(result["migration_state"],"complete")
        self.assertEqual(result["commands_for_review_only"],[])
        self.assertEqual(result["registries_remaining"],[])
        self.assertTrue(result["gate_admin_is_immutable"])

    def test_forged_aggregate_flags_third_party_admin_and_gate_change_fail(self):
        case=fixture()
        case["contracts"][0]["current_administrator"]=NEW
        case["contracts"][0]["matches_deployment_administrator"]=False
        with self.assertRaises(ValueError):plan(case,NEW)
        case["same_administrator_for_all_contracts"]=False
        with self.assertRaises(ValueError):plan(case,NEW)
        case["all_administrators_match_deployment"]=False
        self.assertEqual(plan(case,NEW)["migration_state"],"in-progress")
        case["contracts"][1]["current_administrator"]="GAAACAQDAQCQMBYIBEFAWDANBYHRAEISCMKBKFQXDAMRUGY4DUPB7JZX"
        case["contracts"][1]["matches_deployment_administrator"]=False
        with self.assertRaises(ValueError):plan(case,NEW)
        case=fixture()
        case["contracts"][2]["current_administrator"]=NEW
        case["contracts"][2]["matches_deployment_administrator"]=False
        with self.assertRaises(ValueError):plan(case,NEW)

    def test_gate_immutable_and_staged_other_party_requires_manual_review(self):
        case=fixture()
        case["contracts"][2]["pending_administrator"]=NEW
        with self.assertRaises(ValueError):plan(case,NEW)
        case=fixture()
        case["contracts"][1]["pending_administrator"]="GAAACAQDAQCQMBYIBEFAWDANBYHRAEISCMKBKFQXDAMRUGY4DUPB7JZX"
        with self.assertRaises(ValueError):plan(case,NEW)

if __name__ == "__main__":
    unittest.main()
