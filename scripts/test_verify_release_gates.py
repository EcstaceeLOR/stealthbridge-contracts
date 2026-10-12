import unittest
from verify_release_gates import verify

M = {"schemaVersion":1,"network":"testnet","status":"not-deployed","verified":False,"contractAddresses":{},"assetIssuers":{},"txHashes":[]}
I = {"schemaVersion":1,"network":"testnet","status":"source-interface-only","contracts":{}}
class ReleaseGateTests(unittest.TestCase):
    def test_source_only(self):
        self.assertTrue(verify(M,I))
    def test_reject_deployment_claim(self):
        with self.assertRaises(AssertionError): verify({**M,"status":"deployed"},I)
    def test_reject_verified_claim(self):
        with self.assertRaises(AssertionError): verify({**M,"verified":True},I)
    def test_reject_mainnet(self):
        with self.assertRaises(AssertionError): verify({**M,"network":"public"},I)
    def test_reject_contract_address(self):
        with self.assertRaises(AssertionError): verify({**M,"contractAddresses":{"corridor-registry":"unknown"}},I)
    def test_reject_interface_drift(self):
        with self.assertRaises(AssertionError): verify(M,{**I,"status":"deployed"})
if __name__ == "__main__": unittest.main()
