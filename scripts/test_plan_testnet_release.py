"""No live accounts, network calls, signing or deployments in release plan tests."""
import base64
import binascii
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from plan_testnet_release import initial_plan, upgrade_plan, valid_contract_address, prepared_artifacts

WASM={
    "corridor-registry":"stealthbridge_corridor_registry.wasm",
    "policy-registry":"stealthbridge_policy_registry.wasm",
    "governance-gate":"stealthbridge_governance_gate.wasm",
}

def strkey(version,seed=3):
    raw=bytes([version])+bytes([seed])*32
    checksum=binascii.crc_hqx(raw,0).to_bytes(2,"little")
    return base64.b32encode(raw+checksum).decode()

class OfflineReleasePlanTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.path=Path(self.temp.name)
        hashes={}
        for i,(name,wasm) in enumerate(WASM.items()):
            data=bytes([i+1])*32
            (self.path/wasm).write_bytes(data)
            hashes[name]={"wasmSha256":hashlib.sha256(data).hexdigest()}
        (self.path/"provenance.json").write_text(json.dumps({
            "sdk":"27.0.6","targetProtocol":27,"target":"wasm32v1-none",
            "dirty":False,"reproducibleBuilds":2,"contractArtifacts":hashes
        }))
        self.admin=strkey(48)
        self.contract=strkey(16)

    @patch("plan_testnet_release.verify_artifacts",return_value=3)
    def test_reviewed_initial_plan_never_invokes_or_signs(self,verify):
        result=initial_plan(self.path,"operator_local",self.admin)
        self.assertEqual(result["network"],"testnet")
        self.assertFalse(result["on_chain_verified"])
        self.assertEqual(result["payment_execution"],"disabled")
        self.assertEqual(len(result["commands_for_local_operator_only"]),3)
        self.assertTrue(all("--network testnet" in c for c in result["commands_for_local_operator_only"]))
        self.assertTrue(all("stellar contract deploy" in c for c in result["commands_for_local_operator_only"]))
        self.assertNotIn("secret",str(result["commands_for_local_operator_only"]))
        verify.assert_called_once()

    @patch("plan_testnet_release.verify_artifacts",return_value=3)
    def test_upgrade_plan_requires_existing_valid_c_address(self,_):
        self.assertTrue(valid_contract_address(self.contract))
        self.assertFalse(valid_contract_address(self.admin))
        result=upgrade_plan(
            self.path,"operator_local","policy-registry",self.contract,
            self.path/WASM["policy-registry"],120
        )
        self.assertEqual(result["contract"],"policy-registry")
        self.assertEqual(len(result["commands_for_local_operator_only"]),6)
        self.assertFalse(result["on_chain_verified"])
        self.assertIn("set_paused",str(result["commands_for_local_operator_only"]))
        for invalid in [self.admin,self.contract[:-1]+"A","C"+"A"*55]:
            if invalid==self.contract:continue
            with self.assertRaises(ValueError):
                upgrade_plan(self.path,"operator_local","policy-registry",invalid,
                    self.path/WASM["policy-registry"],120)
        with self.assertRaises(ValueError):
            upgrade_plan(self.path,"operator_local","governance-gate",self.contract,
                self.path/WASM["governance-gate"],120)

    @patch("plan_testnet_release.verify_artifacts",return_value=3)
    def test_release_rejects_secret_like_identity_and_unreviewed_binary(self,_):
        for alias in ["S"+"A"*55,"operator local","../credentials","wallet;curl", ""]:
            with self.assertRaises(ValueError):
                initial_plan(self.path,alias,self.admin)
        (self.path/WASM["policy-registry"]).write_bytes(b"modified WASM")
        with self.assertRaises(ValueError):
            prepared_artifacts(self.path)

    @patch("plan_testnet_release.verify_artifacts",return_value=3)
    def test_invalid_release_or_upgrade_window_rejected(self,_):
        release=json.loads((self.path/"provenance.json").read_text())
        release["dirty"]=True
        (self.path/"provenance.json").write_text(json.dumps(release))
        with self.assertRaises(ValueError):
            initial_plan(self.path,"operator_local",self.admin)
        release["dirty"]=False
        (self.path/"provenance.json").write_text(json.dumps(release))
        for delay in [0,4,20_001]:
            with self.assertRaises(ValueError):
                upgrade_plan(self.path,"operator_local","policy-registry",self.contract,
                    self.path/WASM["policy-registry"],delay)

if __name__=="__main__":
    unittest.main()
