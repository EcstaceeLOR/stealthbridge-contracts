import copy
import json
import sys
import unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
from check_privacy_evidence import validate
DOC=json.loads((ROOT/"integrations/privacy-evidence-vectors.v1.json").read_text())
class PrivateRailEvidenceTests(unittest.TestCase):
 def test_incomplete_proof_plan_does_not_enable_execution(self):
    self.assertEqual(validate(DOC),12)
 def test_claimed_payment_or_cryptographic_audit_safety_rejected(self):
    for key in ("cryptographicSoundness","independentAudit","crossWalletReproduction",
                "publicDisclosureReview","ledgerEvidence","issuerOrPoolAuthorization"):
        changed=copy.deepcopy(DOC);changed["requiredGates"][key]=True
        with self.subTest(key=key),self.assertRaisesRegex(ValueError,"evidence gate"):
            validate(changed)
    changed=copy.deepcopy(DOC);changed["paymentExecutionEnabled"]=True
    with self.assertRaisesRegex(ValueError,"execution"):
        validate(changed)
 def test_crossing_privacy_models_and_replay_vector_edits_rejected(self):
    for key,value in (("privacyIntent","counterparty-hidden"),
                      ("proofSystem","Groth16"),("upstreamCommit","a"*40)):
        changed=copy.deepcopy(DOC);changed["rails"][0][key]=value
        with self.subTest(key=key),self.assertRaisesRegex(ValueError,"conflated"):
            validate(changed)
    changed=copy.deepcopy(DOC);changed["rails"][1]["vectors"][1]["evidence"]="claim"
    with self.assertRaisesRegex(ValueError,"unverified or altered"):
        validate(changed)
 def test_missing_cross_wallet_case_fails(self):
    changed=copy.deepcopy(DOC);changed["rails"][1]["vectors"].pop(0)
    with self.assertRaisesRegex(ValueError,"missing negative"):
        validate(changed)
 def test_public_withdrawal_edge_must_not_be_declared_anonymous(self):
    changed=copy.deepcopy(DOC);changed["rails"][1]["vectors"][4]["expected"]="external-amount-hidden"
    with self.assertRaisesRegex(ValueError,"unverified or altered"):
        validate(changed)
if __name__=="__main__":unittest.main()
