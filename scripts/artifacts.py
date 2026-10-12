#!/usr/bin/env python3
"""Build twice in isolated directories and record offline artifact provenance."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "artifacts"
CLI_VERSION = "27.1.0"


def run(*args, **kwargs):
    return subprocess.check_output(args, cwd=ROOT, **kwargs).decode().strip()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def source_snapshot():
    files = sorted(set(run("git", "ls-files", "--cached", "--others", "--exclude-standard", "-z").split("\0")) - {""})
    source = hashlib.sha256()
    for path in files:
        source.update(path.encode() + b"\0" + (ROOT / path).read_bytes() + b"\0")
    return {
        "sourceCommit": run("git", "rev-parse", "HEAD"),
        "sourceSha256": source.hexdigest(),
        "dirty": bool(run("git", "status", "--porcelain")),
    }


def main():
    cli = os.environ.get("STELLAR", "stellar")
    cli_version = run(cli, "--version")
    if not cli_version.startswith(f"stellar {CLI_VERSION} "):
        raise SystemExit(f"Expected Stellar CLI {CLI_VERSION}: {cli_version}")
    rust = run("rustc", "--version")
    if not rust.startswith("rustc 1.91.0 "):
        raise SystemExit(f"Unexpected compiler: {rust}")
    OUT.mkdir(exist_ok=True)
    source = source_snapshot()
    contracts = {
        "corridor-registry": ("stealthbridge_corridor_registry.wasm", "corridor-registry.abi.json"),
        "policy-registry": ("stealthbridge_policy_registry.wasm", "policy-registry.abi.json"),
        "governance-gate": ("stealthbridge_governance_gate.wasm", "governance-gate.abi.json"),
    }
    builds = []
    for _ in range(2):
        with tempfile.TemporaryDirectory(prefix="registry-build-") as directory:
            env = os.environ.copy()
            env.update(CARGO_TARGET_DIR=directory, CARGO_INCREMENTAL="0",
                       RUSTFLAGS=f"--remap-path-prefix={ROOT}=/source --remap-path-prefix={directory}=/target")
            subprocess.run(["cargo", "build", "--locked", "--release", "--target", "wasm32v1-none",
                            "--workspace"], cwd=ROOT, env=env, check=True)
            artifacts = {
                name: (Path(directory) / "wasm32v1-none/release" / wasm_name).read_bytes()
                for name, (wasm_name, _) in contracts.items()
            }
            builds.append(artifacts)
    for name in contracts:
        if builds[0][name] != builds[1][name]:
            raise SystemExit(f"Isolated WASM builds differ for {name}")

    digests = {}
    files = []
    for name, (wasm_name, abi_name) in contracts.items():
        wasm = OUT / wasm_name
        wasm.write_bytes(builds[0][name])
        abi = run(cli, "contract", "info", "interface", "--wasm", str(wasm), "--output", "json") + "\n"
        abi_path = OUT / abi_name
        abi_path.write_text(abi)
        files.extend((wasm, abi_path))
        digests[name] = {
            "wasmSha256": sha(builds[0][name]),
            "abiSha256": sha(abi.encode()),
            "wasmBytes": len(builds[0][name])
        }
    if source_snapshot() != source:
        raise SystemExit("Source changed during artifact generation; rebuild a stable checkout")
    corridor = digests["corridor-registry"]
    metadata = {
        "schemaVersion": 1, "artifactVersion": "corridor-registry/0.1.0",
        **source,
        "wasmSha256": corridor["wasmSha256"], "abiSha256": corridor["abiSha256"],
        "wasmBytes": corridor["wasmBytes"], "contractArtifacts": digests,
        "isolatedBuildDigests": {name: [sha(builds[0][name]), sha(builds[1][name])]
                                 for name in contracts},
        "rustc": rust, "stellarCli": cli_version,
        "sdk": "27.0.6", "sdkMajor": 27, "targetProtocol": 27,
        "target": "wasm32v1-none", "reproducibleBuilds": 2,
        "cargoLockSha256": sha((ROOT / "Cargo.lock").read_bytes()),
    }
    (OUT / "provenance.json").write_text(json.dumps(metadata, indent=2) + "\n")
    files.append(OUT / "provenance.json")
    (OUT / "SHA256SUMS").write_text("".join(
        f"{sha(path.read_bytes())}  {path.name}\n" for path in files
    ))
    print(json.dumps(metadata, indent=2))


if __name__ == "__main__":
    main()
