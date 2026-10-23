import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from verify_privacy_testnet_receipts import inspect_receipt, ReceiptError

TX = "ab" * 32
URL = "https://rpc.example.test"


class TestTestnetReceiptInspection(unittest.TestCase):
    def test_readonly_success_is_not_privacy_attestation(self):
        calls = []
        def rpc(url, method, params):
            calls.append((url, method, params))
            if method == "getNetwork":
                return {"passphrase": "Test SDF Network ; September 2015"}
            return {"hash": TX, "status": "SUCCESS", "ledger": 123456}

        report = inspect_receipt(TX, "SUCCESS", URL, rpc=rpc)
        self.assertTrue(report["receiptChecked"])
        self.assertFalse(report["confirmedBySecondRpc"])
        for key in ("privacyProven", "zkProofVerified",
                    "walletLinkabilityReviewed", "paymentEnabled"):
            self.assertIs(report[key], False)
        self.assertEqual(len(calls), 2)
        self.assertEqual(calls[1][2], {"hash": TX})

    def test_rejects_false_failed_proof_claim_or_missing_ledger(self):
        def rpc(url, method, params):
            if method == "getNetwork":
                return {"passphrase": "Test SDF Network ; September 2015"}
            return {"hash": TX, "status": "SUCCESS", "ledger": 10}
        with self.assertRaisesRegex(ReceiptError, "status"):
            inspect_receipt(TX, "FAILED", URL, rpc=rpc)
        def wrong_ledger(url, method, params):
            if method == "getNetwork":
                return {"passphrase": "Test SDF Network ; September 2015"}
            return {"hash": TX, "status": "FAILED", "ledger": None}
        with self.assertRaisesRegex(ReceiptError, "ledger"):
            inspect_receipt(TX, "FAILED", URL, rpc=wrong_ledger)

    def test_rejects_wrong_hash_network_or_rpc_auth(self):
        def mismatch(url, method, params):
            if method == "getNetwork":
                return {"passphrase": "Public Global Stellar Network ; September 2015"}
            return {"hash": TX, "status": "SUCCESS", "ledger": 1234}
        with self.assertRaisesRegex(ReceiptError, "not Stellar Testnet"):
            inspect_receipt(TX, "SUCCESS", URL, rpc=mismatch)
        def bad_hash(url, method, params):
            if method == "getNetwork":
                return {"passphrase": "Test SDF Network ; September 2015"}
            return {"hash": "ff" * 32, "status": "SUCCESS", "ledger": 1234}
        with self.assertRaisesRegex(ReceiptError, "different"):
            inspect_receipt(TX, "SUCCESS", URL, rpc=bad_hash)
        for bad_url in ("http://rpc.example.test", "https://user:secret@rpc.example.test"):
            with self.subTest(url=bad_url), self.assertRaisesRegex(ReceiptError, "HTTPS"):
                inspect_receipt(TX, "SUCCESS", bad_url, rpc=bad_hash)
        with self.assertRaisesRegex(ReceiptError, "64-hex"):
            inspect_receipt("not-a-hash", "SUCCESS", URL, rpc=bad_hash)

    def test_independent_provider_confirmation_is_still_not_a_zk_proof(self):
        calls = []
        def rpc(url, method, params):
            calls.append((url, method))
            if method == "getNetwork":
                return {"passphrase": "Test SDF Network ; September 2015"}
            return {"hash": TX, "status": "SUCCESS", "ledger": 321}
        result = inspect_receipt(
            TX, "SUCCESS", URL, rpc=rpc,
            confirm_rpc_url="https://different.example.test"
        )
        self.assertTrue(result["confirmedBySecondRpc"])
        self.assertIs(result["zkProofVerified"], False)
        self.assertEqual(len(calls), 4)
        self.assertEqual(calls[2][0], "https://different.example.test")

    def test_second_provider_cannot_substitute_wrong_ledger_or_network(self):
        def rpc(url, method, params):
            if method == "getNetwork":
                return {"passphrase": "Test SDF Network ; September 2015"}
            return {"hash": TX, "status": "SUCCESS",
                    "ledger": 321 if url == URL else 322}
        with self.assertRaisesRegex(ReceiptError, "disagree"):
            inspect_receipt(
                TX, "SUCCESS", URL, rpc=rpc,
                confirm_rpc_url="https://different.example.test"
            )
        with self.assertRaisesRegex(ReceiptError, "separate HTTPS"):
            inspect_receipt(TX, "SUCCESS", URL, rpc=rpc, confirm_rpc_url=URL)

    def test_report_cannot_include_secrets_from_rpc(self):
        def rpc(url, method, params):
            if method == "getNetwork":
                return {"passphrase": "Test SDF Network ; September 2015"}
            return {"hash": TX, "status": "SUCCESS", "ledger": 42,
                    "envelopeXdr": "SHOULD_NOT_LEAK", "privateWitness": "SECRET"}
        report = inspect_receipt(TX, "SUCCESS", URL, rpc=rpc)
        self.assertNotIn("SECRET", repr(report))
        self.assertNotIn("SHOULD_NOT_LEAK", repr(report))


if __name__ == "__main__":
    unittest.main()
