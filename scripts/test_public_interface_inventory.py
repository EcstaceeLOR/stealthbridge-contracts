"""Smoke and mutation tests for the source-only Rust ABI surface."""
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "integrations/public-soroban-interface.v1.json"
CHECK = ROOT / "scripts/verify_public_interface.py"

class PublicSorobanInventoryTests(unittest.TestCase):
    def setUp(self):
        self.workdir = tempfile.TemporaryDirectory(prefix="contract-interface-fixture-")
        self.addCleanup(self.workdir.cleanup)
        self.path = Path(self.workdir.name)
        metadata = json.loads(SOURCE.read_text())
        (self.path / "integrations").mkdir()
        (self.path / "integrations/public-soroban-interface.v1.json").write_text(
            json.dumps(metadata) + "\n")
        for record in metadata["contracts"].values():
            source_path = record["source"]
            output = self.path / source_path
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_text((ROOT / source_path).read_text())
        self.metadata = metadata

    def check(self):
        return subprocess.run([sys.executable, str(CHECK)], cwd=self.path,
                              capture_output=True, text=True)

    def test_current_rust_sources_match_declared_inventory(self):
        result = self.check()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_unreviewed_public_rust_method_breaks_ci(self):
        source = self.path / self.metadata["contracts"]["corridor-registry"]["source"]
        source.write_text(source.read_text() + "\npub fn surprise_payout(env: Env) -> bool { false }\n")
        self.assertNotEqual(self.check().returncode, 0)

    def test_renamed_public_read_breaks_ci(self):
        source = self.path / self.metadata["contracts"]["policy-registry"]["source"]
        source.write_text(source.read_text().replace(
            "pub fn is_effective_commitment(", "pub fn renamed_effective_commitment(", 1))
        self.assertNotEqual(self.check().returncode, 0)

if __name__ == "__main__":
    unittest.main()
