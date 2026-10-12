#!/usr/bin/env python3
"""Offline fail-closed release guard; never deploys or signs."""
import json
from pathlib import Path

def verify(manifest, interface):
    assert manifest.get("schemaVersion") == 1
    assert manifest.get("network") == "testnet"
    assert manifest.get("status") == "not-deployed"
    assert manifest.get("verified") is False
    assert manifest.get("contractAddresses") == {}
    assert manifest.get("assetIssuers") == {}
    assert manifest.get("txHashes") == []
    assert interface.get("schemaVersion") == 1
    assert interface.get("network") == "testnet"
    assert interface.get("status") == "source-interface-only"
    assert isinstance(interface.get("contracts"), dict)
    return True

if __name__ == "__main__":
    manifest = json.loads(Path("deployments/testnet/manifest.json").read_text())
    interface = json.loads(Path("integrations/public-soroban-interface.v1.json").read_text())
    verify(manifest, interface)
    print("PASS: undeployed source-only release boundary")
