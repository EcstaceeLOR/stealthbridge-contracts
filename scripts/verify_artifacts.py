#!/usr/bin/env python3
"""Verify every reproducible source-only Soroban WASM/ABI build artifact.

This checks offline file integrity and does NOT verify Testnet deployment,
signature authorization, governance privileges, or confidential payments.
"""
import argparse
import hashlib
import json
from pathlib import Path

CONTRACT_FILES = {
    "corridor-registry": ("stealthbridge_corridor_registry.wasm", "corridor-registry.abi.json"),
    "policy-registry": ("stealthbridge_policy_registry.wasm", "policy-registry.abi.json"),
    "governance-gate": ("stealthbridge_governance_gate.wasm", "governance-gate.abi.json"),
}


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def verify(root):
    root = Path(root)
    provenance_bytes = (root / "provenance.json").read_bytes()
    evidence = json.loads(provenance_bytes)
    if not isinstance(evidence, dict) or evidence.get("dirty") is not False:
        raise ValueError("artifact evidence must come from a clean source tree")
    if evidence.get("target") != "wasm32v1-none" or evidence.get("reproducibleBuilds") != 2:
        raise ValueError("only twice-reproduced Soroban WASM is acceptable")
    if evidence.get("sdk") != "27.0.6" or evidence.get("targetProtocol") != 27:
        raise ValueError("Soroban compiler protocol metadata is incompatible")

    artifacts = evidence.get("contractArtifacts")
    if not isinstance(artifacts, dict) or set(artifacts) != set(CONTRACT_FILES):
        raise ValueError("all three contract artifacts must be present, with no extra names")

    isolated = evidence.get("isolatedBuildDigests")
    if not isinstance(isolated, dict) or set(isolated) != set(CONTRACT_FILES):
        raise ValueError("isolated WASM digest pairs missing")
    for name in CONTRACT_FILES:
        pair = isolated[name]
        if not isinstance(pair, list) or len(pair) != 2 or any(
                not isinstance(item, str) or len(item) != 64 for item in pair):
            raise ValueError(f"invalid reproducibility hash pair for {name}")
        if pair[0] != pair[1] or pair[0] != artifacts[name]["wasmSha256"]:
            raise ValueError(f"isolated builds disagree for {name}")
    digests = {}
    for name, (wasm_name, abi_name) in CONTRACT_FILES.items():
        metadata = artifacts[name]
        if not isinstance(metadata, dict) or set(metadata) != {"wasmSha256", "abiSha256", "wasmBytes"}:
            raise ValueError(f"invalid provenance for {name}")
        wasm = (root / wasm_name).read_bytes()
        abi = (root / abi_name).read_bytes()
        if len(wasm) == 0 or len(abi) == 0:
            raise ValueError(f"empty WASM/ABI for {name}")
        if len(wasm) != metadata["wasmBytes"]:
            raise ValueError(f"WASM size mismatch for {name}")
        if sha256(wasm) != metadata["wasmSha256"] or sha256(abi) != metadata["abiSha256"]:
            raise ValueError(f"WASM/ABI digest mismatch for {name}")
        digests[wasm_name] = sha256(wasm)
        digests[abi_name] = sha256(abi)

    if evidence.get("wasmSha256") != artifacts["corridor-registry"]["wasmSha256"]:
        raise ValueError("legacy corridor WASM hash differs from evidence")
    if evidence.get("abiSha256") != artifacts["corridor-registry"]["abiSha256"]:
        raise ValueError("legacy corridor ABI hash differs from evidence")
    if evidence.get("wasmBytes") != artifacts["corridor-registry"]["wasmBytes"]:
        raise ValueError("legacy corridor size differs from evidence")
    digests["provenance.json"] = sha256(provenance_bytes)
    expected = [f"{digest}  {name}" for name, digest in digests.items()]
    actual = (root / "SHA256SUMS").read_text().splitlines()
    if actual != expected:
        raise ValueError("SHA256SUMS does not match exactly the expected files and order")
    return len(CONTRACT_FILES)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artifacts", type=Path, default=Path("artifacts"))
    args = parser.parse_args()
    try:
        count = verify(args.artifacts)
    except (ValueError, OSError, TypeError, KeyError, json.JSONDecodeError) as error:
        parser.exit(1, f"Artifact verification failed: {error}\n")
    print(f"PASS: {count} local Soroban WASM and ABI artifacts match pinned provenance; no deployment asserted.")


if __name__ == "__main__":
    main()
