import copy
import json
import sys
import unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
from check_issuer_capabilities import validate
RECORD=json.loads((ROOT/"integrations/confidential-tokens/capability-review.v1.json").read_text())
class IssuerCapabilities(unittest.TestCase):
    def test_verified_source_inventory_only(self):
        self.assertEqual(validate(RECORD),10)
    def test_no_payment_or_issuer_claim(self):
        for field in ("privacyVerified","paymentExecutionEnabled","issuerAgreementVerified"):
            record=copy.deepcopy(RECORD)
            record["local"][field]=True
            with self.subTest(field=field),self.assertRaisesRegex(ValueError,"unverified issuer"):
                validate(record)
    def test_confidential_counterparties_and_public_edges_cannot_be_fabricated(self):
        for key,value in (("counterpartyAddresses","hidden"),
                          ("depositWithdrawAmounts","hidden"),
                          ("privateTransferAmounts","verified-private"),
                          ("issuerRedemption","onchain-guaranteed")):
            record=copy.deepcopy(RECORD)
            record["disclosures"][key]=value
            with self.subTest(key=key),self.assertRaisesRegex(ValueError,"privacy disclosure"):
                validate(record)
    def test_unreviewed_upstream_version_breaks_pin(self):
        record=copy.deepcopy(RECORD)
        record["upstream"]["sorobanSdk"]="27.0.6"
        with self.assertRaisesRegex(ValueError,"unreviewed upstream"):
            validate(record)
    def test_fake_testnet_receipt_and_missing_mint_policy_rejected(self):
        for change in ({"stealthbridge":"verified"},{"testnetReceipt":"a"*64},
                       {"externalAudit":"approved"}):
            record=copy.deepcopy(RECORD)
            record["capabilities"][0].update(change)
            with self.subTest(change=change),self.assertRaisesRegex(ValueError,"fail closed"):
                validate(record)
        record=copy.deepcopy(RECORD)
        record["capabilities"].pop()
        with self.assertRaisesRegex(ValueError,"incomplete issuer"):
            validate(record)
if __name__=="__main__":unittest.main()
