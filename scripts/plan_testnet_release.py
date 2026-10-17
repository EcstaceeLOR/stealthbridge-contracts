#!/usr/bin/env python3
"""OFFLINE release plan for reviewed StealthBridge Testnet governance contracts.

Never signs, uploads, invokes, funds or submits a transaction. Only local
source/WASM verification and printed operator commands. No private keys.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shlex
import sys

from preflight_testnet import valid_operator_address
from verify_artifacts import verify as verify_artifacts

ROOT=Path(__file__).resolve().parents[1]
CONTRACT_FILES={
    "corridor-registry":"stealthbridge_corridor_registry.wasm",
    "policy-registry":"stealthbridge_policy_registry.wasm",
    "governance-gate":"stealthbridge_governance_gate.wasm",
}
ALIAS=re.compile(r"^[a-z][a-z0-9_-]{1,31}$")
HASH=re.compile(r"^[a-f0-9]{64}$")
CONTRACT=re.compile(r"^C[A-Z2-7]{55}$")

def safe_alias(value):
    if not isinstance(value,str) or not ALIAS.fullmatch(value):
        raise ValueError("Use a named local Stellar CLI identity, not a secret key")
    return value

def digest(path):
    h=hashlib.sha256()
    with path.open("rb") as stream:
        while chunk:=stream.read(128*1024):
            h.update(chunk)
    return h.hexdigest()

def prepared_artifacts(directory):
    directory=Path(directory)
    if verify_artifacts(directory)!=3:
        raise ValueError("Require three checked reproducible release artifacts")
    provenance=json.loads((directory/"provenance.json").read_text())
    if provenance.get("sdk")!="27.0.6" or provenance.get("targetProtocol")!=27 or provenance.get("target")!="wasm32v1-none":
        raise ValueError("Unexpected Soroban SDK, WASM target or intended protocol")
    if provenance.get("dirty") is not False or provenance.get("reproducibleBuilds")!=2:
        raise ValueError("Deploy only two identical artifacts from a clean reviewed checkout")
    published=provenance["contractArtifacts"]
    checks={}
    for name,filename in CONTRACT_FILES.items():
        path=directory/filename
        value=digest(path)
        if not HASH.fullmatch(value) or published[name]["wasmSha256"]!=value:
            raise ValueError("WASM hash does not match reviewed release provenance")
        checks[name]=value
    return checks

def initial_plan(artifacts,identity,admin):
    safe_alias(identity)
    if not valid_operator_address(admin):
        raise ValueError("Admin must be a valid public Stellar G account")
    hashes=prepared_artifacts(artifacts)
    a=shlex.quote(identity); g=shlex.quote(admin); directory=Path(artifacts)
    commands=[]
    for name in ["corridor-registry","policy-registry"]:
        filename=shlex.quote(str(directory/CONTRACT_FILES[name]))
        commands.append(f"stellar contract deploy --wasm {filename} --source-account {a} --network testnet -- --admin {g}")
    gate=shlex.quote(str(directory/CONTRACT_FILES["governance-gate"]))
    commands.append(
       f"stellar contract deploy --wasm {gate} --source-account {a} --network testnet -- --admin {g} --corridor_registry <CORRIDOR_CONTRACT_ID> --policy_registry <POLICY_CONTRACT_ID>"
    )
    return {
      "mode":"initial-deployment-review", "network":"testnet",
      "payment_execution":"disabled", "on_chain_verified":False,
      "artifact_hashes":hashes, "commands_for_local_operator_only":commands,
      "checks_before_signing":[
        "Validate Stellar Testnet passphrase, ledger time, current protocol and account funding",
        "Review original clean source commit, reproducible WASM and public constructor admin",
        "Use locally held Stellar CLI identity or hardware wallet; never disclose seed or private key",
        "Save signed transaction hashes and contract IDs; independently re-read deployed bytecode hash",
        "Run source/ABI and constructor-state compatibility checks before changing deployment evidence",
      ]
    }

def upgrade_plan(artifacts, identity, contract_name, contract_id, new_wasm,delay):
    safe_alias(identity)
    if contract_name not in ("corridor-registry","policy-registry"):
        raise ValueError("Only pausable registries support in-place WASM upgrade")
    if not CONTRACT.fullmatch(contract_id):
        raise ValueError("Pass the verified public on-chain C-address of this contract")
    if not isinstance(delay,int) or not (5<=delay<=20_000):
        raise ValueError("Upgrade delay must be 5 to 20,000 ledgers")
    digests=prepared_artifacts(artifacts)
    wasm=Path(new_wasm)
    value=digest(wasm)
    if value!=digests[contract_name]:
        raise ValueError("New WASM file differs from independently verified release artifact")
    quote=lambda value: shlex.quote(str(value))
    common=f"--id {quote(contract_id)} --source-account {quote(identity)} --network testnet"
    return {
      "mode":"offline-upgrade-review-only","network":"testnet",
      "payment_execution":"disabled","contract":contract_name,
      "candidate_wasm_sha256":value,"on_chain_verified":False,
      "commands_for_local_operator_only":[
        f"stellar contract upload --wasm {quote(wasm)} --source-account {quote(identity)} --network testnet",
        f"stellar contract invoke {common} -- set_paused --paused true",
        f"stellar contract invoke {common} -- propose_upgrade --wasm_hash {value} --delay_ledgers {delay}",
        f"stellar contract invoke {common} -- pending_upgrade",
        f"stellar contract invoke {common} -- execute_upgrade --wasm_hash {value}",
        f"stellar contract info hash --contract-id {quote(contract_id)} --network testnet",
      ],
      "checks_before_signing":[
        "Do not run on an unverified or unrecognized contract address",
        "Verify contract admin, paused state, current WASM hash and reviewed storage compatibility",
        "Do not execute before the on-chain ready_at_ledger or after expires_at_ledger",
        "Independently match uploaded WASM hash and candidate ABI to audited source",
        "Get separately signed operator approval; CLI invokes may only simulate unless explicitly sent",
        "After execution recheck same contract address, new WASM hash, admin, storage and continued pause",
        "If verification fails remain paused and cancel any still-pending proposal",
      ]
    }

def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artifacts",type=Path,default=ROOT/"artifacts")
    parser.add_argument("--identity",help="Name of locally configured Stellar CLI signing identity")
    parser.add_argument("--admin",help="Public G address of administrator; initial deployments only")
    parser.add_argument("--upgrade-contract",choices=["corridor-registry","policy-registry"])
    parser.add_argument("--contract-id",help="Verified public C address; upgrades only")
    parser.add_argument("--new-wasm",type=Path,help="Reviewed built release WASM; upgrades only")
    parser.add_argument("--delay-ledgers",type=int,default=120)
    args=parser.parse_args(argv)
    try:
        if not args.identity: raise ValueError("Local signing identity name is required")
        upgrading=args.upgrade_contract is not None
        if upgrading:
            if args.admin or not args.contract_id or not args.new_wasm:
                raise ValueError("Upgrade requires reviewed contract ID and WASM, not an admin seed")
            plan=upgrade_plan(args.artifacts,args.identity,args.upgrade_contract,
                              args.contract_id,args.new_wasm,args.delay_ledgers)
        else:
            if args.contract_id or args.new_wasm or not args.admin:
                raise ValueError("Initial deploy needs only local identity name and public admin address")
            plan=initial_plan(args.artifacts,args.identity,args.admin)
    except (OSError, KeyError, TypeError, ValueError, json.JSONDecodeError):
        print("Deployment planning rejected: check public IDs, clean artifacts and signatures locally. No transaction sent.",file=sys.stderr)
        return 1
    print(json.dumps(plan,indent=2))
    return 0

if __name__=="__main__":
    raise SystemExit(main())
