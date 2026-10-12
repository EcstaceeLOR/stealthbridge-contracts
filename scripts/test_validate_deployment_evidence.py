import copy
import json
import sys
import unittest
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
from validate_deployment_evidence import validate_evidence

E=json.loads((ROOT/"deployments/testnet/evidence-chain.v1.json").read_text())
M=json.loads((ROOT/"deployments/testnet/manifest.json").read_text())

class DeploymentEvidenceGateTests(unittest.TestCase):
    def test_source_only_manifest_and_pending_chain(self):
        self.assertEqual(validate_evidence(E,M),6)
    def test_mainnet_or_fake_passphrase_rejected(self):
        for mutation in ({"network":"public"},{"networkPassphrase":"wrong"}):
            with self.subTest(mutation=mutation),self.assertRaisesRegex(ValueError,"wrong Stellar network"):
                validate_evidence({**E,**mutation},M)
    def test_deployed_claims_cannot_skip_attestation(self):
        for mutation in ({"releaseStatus":"deployed"},{"privacyVerified":True},
                         {"paymentExecutionEnabled":True}):
            with self.subTest(mutation=mutation),self.assertRaisesRegex(ValueError,"verification claims"):
                validate_evidence({**E,**mutation},M)
    def test_fake_transaction_address_or_operator_signature_rejected(self):
        for mutation in ({"deploymentTransactionHashes":["a"*64]},
                         {"contractAddresses":{"corridor-registry":"fake"}},
                         {"operatorStatementSignature":"signed-fake"},
                         {"rollbackPauseEvidence":{"paused":True}}):
            with self.subTest(mutation=mutation),self.assertRaises(ValueError):
                validate_evidence({**E,**mutation},M)
    def test_partial_and_premature_gates_rejected(self):
        for change in ({"status":"passed"},{"receiptSha256":"a"*64},{"gate":"waived"}):
            data=copy.deepcopy(E)
            data["requiredEvidence"][2].update(change)
            with self.subTest(change=change),self.assertRaisesRegex(ValueError,"pending"):
                validate_evidence(data,M)
        data=copy.deepcopy(E)
        data["requiredEvidence"].pop()
        with self.assertRaisesRegex(ValueError,"incomplete"):
            validate_evidence(data,M)
    def test_future_deployed_manifest_is_not_mistaken_for_verified_record(self):
        with self.assertRaisesRegex(ValueError,"deployed manifest"):
            validate_evidence(E,{**M,"status":"deployed","verified":True})
    def test_unexpected_fields_fail_closed(self):
        with self.assertRaisesRegex(ValueError,"missing or extra"):
            validate_evidence({**E,"activatedAtLedger":123},M)

if __name__=="__main__":unittest.main()
