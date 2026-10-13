import copy
import json
import sys
import unittest
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
from attest_privacy_release import assess,UnsafePrivacyRelease

def read(path):
    return json.loads((ROOT/path).read_text())

DOCUMENTS=(
    "deployments/testnet/manifest.json",
    "integrations/public-soroban-interface.v1.json",
    "integrations/privacy-evidence-vectors.v1.json",
    "integrations/confidential-tokens/capability-review.v1.json",
    "integrations/upstream-source-lock.v1.json",
)
class ReleaseGateTests(unittest.TestCase):
 def setUp(self):
    self.inputs=[read(path) for path in DOCUMENTS]

 def test_only_no_go_is_permitted_without_independent_proof(self):
    report=assess(*self.inputs)
    self.assertEqual(report["decision"],"NO_GO")
    self.assertIs(report["paymentExecutionEnabled"],False)
    self.assertIs(report["contractDeploymentVerified"],False)
    self.assertEqual(set(report["rails"]),{"confidential-token","private-payments"})
    self.assertEqual(len(report["blockers"]),6)
    self.assertTrue(all(row["verified"] is False for row in report["rails"].values()))
    self.assertTrue(all(row["status"]=="missing-independent-evidence" for row in report["blockers"]))

 def test_fake_deployment_claim_is_rejected(self):
    cases=[
        ("status","deployed"),
        ("verified",True),
        ("contractAddresses",{"fake":"C"+"A"*55}),
        ("assetIssuers",{"fake":"G"+"A"*55}),
        ("txHashes",["0"*64]),
        ("network","public"),
    ]
    for key,value in cases:
      inputs=copy.deepcopy(self.inputs);inputs[0][key]=value
      with self.subTest(key=key),self.assertRaisesRegex(UnsafePrivacyRelease,"deployment"):
        assess(*inputs)

 def test_unreviewed_interface_and_payment_claim_are_rejected(self):
    inputs=copy.deepcopy(self.inputs)
    inputs[1]["status"]="deployed"
    with self.assertRaisesRegex(UnsafePrivacyRelease,"interface"):
      assess(*inputs)
    for index,key in [(2,"paymentExecutionEnabled"),(4,"paymentExecutionEnabled")]:
      inputs=copy.deepcopy(self.inputs);inputs[index][key]=True
      with self.subTest(index=index),self.assertRaises(UnsafePrivacyRelease):
        assess(*inputs)
    inputs=copy.deepcopy(self.inputs);inputs[3]["local"]["privacyVerified"]=True
    with self.assertRaises(UnsafePrivacyRelease):
      assess(*inputs)

 def test_no_cross_wallet_or_audit_flag_may_be_set_without_evidence(self):
    for gate in (
      "cryptographicSoundness","independentAudit","crossWalletReproduction",
      "publicDisclosureReview","ledgerEvidence","issuerOrPoolAuthorization"
    ):
      inputs=copy.deepcopy(self.inputs);inputs[2]["requiredGates"][gate]=True
      with self.subTest(gate=gate),self.assertRaises(UnsafePrivacyRelease):
        assess(*inputs)

 def test_missing_or_added_unreviewed_source_or_proof_case_fails(self):
    inputs=copy.deepcopy(self.inputs)
    inputs[4]["sources"].pop()
    with self.assertRaises(UnsafePrivacyRelease):
      assess(*inputs)
    inputs=copy.deepcopy(self.inputs)
    inputs[2]["rails"][0]["vectors"][0]["evidence"]={"fabricatedTx":"0"*64}
    with self.assertRaises(UnsafePrivacyRelease):
      assess(*inputs)

 def test_wrong_network_and_conflated_primitive_fails(self):
    inputs=copy.deepcopy(self.inputs);inputs[2]["rails"][1]["proofSystem"]="UltraHonk"
    with self.assertRaises(UnsafePrivacyRelease):
      assess(*inputs)
    inputs=copy.deepcopy(self.inputs);inputs[2]["networkPassphrase"]="Public Global Stellar Network ; September 2015"
    with self.assertRaises(UnsafePrivacyRelease):
      assess(*inputs)
if __name__=="__main__":unittest.main()
