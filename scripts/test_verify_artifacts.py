"""Negative and positive offline artifact verification tests with synthetic bytes."""
import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from verify_artifacts import CONTRACT_FILES, verify


def digest(data):
    return hashlib.sha256(data).hexdigest()


class VerifyArtifactsTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="stealthbridge-artifact-fixture-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        digests = {}
        contents = {}
        for name, (wasm, abi) in CONTRACT_FILES.items():
            wasm_bytes = b"synthetic-test-only-wasm:" + name.encode()
            abi_bytes = b'{"fixture":"not-a-deployment","name":"' + name.encode() + b'"}\n'
            contents[wasm], contents[abi] = wasm_bytes, abi_bytes
            digests[name] = {
                "wasmSha256": digest(wasm_bytes),
                "abiSha256": digest(abi_bytes),
                "wasmBytes": len(wasm_bytes),
            }
        evidence = {
            "dirty": False, "sdk": "27.0.6", "targetProtocol": 27,
            "target": "wasm32v1-none", "reproducibleBuilds": 2,
            "contractArtifacts": digests,
            "isolatedBuildDigests": {name: [artifact["wasmSha256"]]*2
                                     for name, artifact in digests.items()},
            **digests["corridor-registry"],
        }
        contents["provenance.json"] = (json.dumps(evidence) + "\n").encode()
        for name, data in contents.items():
            (self.root / name).write_bytes(data)
        (self.root / "SHA256SUMS").write_text("".join(
            f"{digest(data)}  {name}\n" for name, data in contents.items()
        ))

    def test_all_three_contracts_verified(self):
        self.assertEqual(verify(self.root), 3)

    def test_tampered_gate_bytecode_rejected(self):
        path = self.root / "stealthbridge_governance_gate.wasm"
        path.write_bytes(path.read_bytes() + b"tampering")
        with self.assertRaisesRegex(ValueError, "WASM size mismatch"):
            verify(self.root)

    def test_tampered_policy_abi_rejected(self):
        path = self.root / "policy-registry.abi.json"
        path.write_bytes(path.read_bytes().replace(b"not-a-deployment", b"not-a-deployMent"))
        with self.assertRaisesRegex(ValueError, "digest mismatch"):
            verify(self.root)

    def test_checksum_manifest_cannot_omit_files(self):
        (self.root / "SHA256SUMS").write_text("only one file\n")
        with self.assertRaisesRegex(ValueError, "SHA256SUMS"):
            verify(self.root)

    def test_second_isolated_build_hash_drift_rejected(self):
        path = self.root / "provenance.json"
        data = json.loads(path.read_text())
        data["isolatedBuildDigests"]["governance-gate"][1] = "0" * 64
        path.write_text(json.dumps(data))
        with self.assertRaisesRegex(ValueError, "isolated builds disagree"):
            verify(self.root)

    def test_reproducibility_hash_evidence_required(self):
        path = self.root / "provenance.json"
        data = json.loads(path.read_text())
        del data["isolatedBuildDigests"]
        path.write_text(json.dumps(data))
        with self.assertRaisesRegex(ValueError, "digest pairs missing"):
            verify(self.root)

    def test_unreproducible_metadata_rejected(self):
        path = self.root / "provenance.json"
        data = json.loads(path.read_text())
        data["reproducibleBuilds"] = 1
        path.write_text(json.dumps(data))
        with self.assertRaisesRegex(ValueError, "twice-reproduced"):
            verify(self.root)


if __name__ == "__main__":
    unittest.main()
