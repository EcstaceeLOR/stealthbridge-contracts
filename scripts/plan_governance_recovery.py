#!/usr/bin/env python3
"""Offline review-only administrator handover plan for Testnet registries.

Input is a previously saved JSON response from the Rust read-only governance
administrator endpoint, NOT a signed proof of current chain state. Never
connect to a wallet, sign, invoke a contract or execute CLI as this script.
"""
import argparse
import base64
import binascii
import json
import re
from pathlib import Path
from time import time

RELEASE_ADMIN = "GC3YRVLB7IN7SNUYVWFUC6OD3QWJXCUZ4HOBNIPVGY7BZDVOYMDNX5TN"
CONTRACTS = (
    ("corridor-registry", "CDS6HBE6YJSZTRG3VR2HLCNHWJZMKB43BLFFEHEH3Z76WXAF46LW7JOB", "new_admin"),
    ("policy-registry", "CBXWEA5PMDTBN5AN7FJYGGAPIWLRKXYHWLVWG5WOXH3HSRDB4RYTJBN5", "successor"),
    ("governance-gate", "CD2KIMDRYJMKGO5SMIBOQP6ADPW5I4THNZFGXWYLIPGSXAI5NRXVAPKV", None),
)
SOURCE = "stellar-rpc/getLedgerEntries/contract-instance-storage"

def public_account(value):
    if not isinstance(value, str) or not re.fullmatch(r"G[A-Z2-7]{55}", value):
        return False
    try:
        raw = base64.b32decode(value)
    except (ValueError, binascii.Error):
        return False
    return (len(raw) == 35 and raw[0] == 48 and
            binascii.crc_hqx(raw[:-2], 0).to_bytes(2, "little") == raw[-2:])

def require(condition, message):
    if not condition:
        raise ValueError(message)

def plan(observation, successor):
    """Pure validation and rendering. No network or filesystem operations."""
    require(public_account(successor), "successor needs a checksum-valid classic G address")
    require(successor != RELEASE_ADMIN, "new administrator must differ from original")
    require(isinstance(observation, dict) and observation.get("schema_version") == 1,
            "invalid governance observation envelope")
    require(observation.get("network") == "testnet" and observation.get("source") == SOURCE,
            "only read-only Testnet instance-storage observations can be reviewed")
    for field in ("reviewed_release_executable_hashes_confirmed",
                  "administrator_entries_read_from_current_storage",
                  "same_administrator_for_all_contracts",
                  "all_administrators_match_deployment"):
        require(observation.get(field) is True, "all three reviewed-code ownership gates are required")
    for field in ("payment_execution_enabled", "fee_collection_enabled",
                  "privacy_verified", "signer_possession_verified"):
        require(observation.get(field) is False, "a fee, payment, privacy or signer claim is forbidden")
    require(observation.get("release_administrator") == RELEASE_ADMIN,
            "release administrator differs from verified deployment")
    # Saved administrator JSON is not signed chain evidence. Even review-only
    # commands must be based on a recently verified Testnet head.
    close = observation.get("observed_ledger_closed_at_unix")
    require(isinstance(close, str) and close.isascii() and close.isdigit() and
            1 <= len(close) <= 20, "ledger close timestamp missing or malformed")
    close_unix = int(close)
    now_unix = int(time())
    require(close_unix > 0 and -30 <= now_unix-close_unix <= 180,
            "administrator observation is stale or future-dated; refresh Testnet RPC")
    ledger = observation.get("observed_ledger_sequence")
    require(type(ledger) is int and ledger >= 5244293, "missing tested ledger sequence")
    entries = observation.get("contracts")
    require(isinstance(entries, list) and len(entries) == 3,
            "expected three unique deployed governance entries")
    commands = []
    for index, (name, cid, parameter) in enumerate(CONTRACTS):
        record = entries[index]
        require(isinstance(record, dict) and record.get("name") == name and
                record.get("contract_id") == cid and
                record.get("current_administrator") == RELEASE_ADMIN and
                record.get("matches_deployment_administrator") is True,
                "contract address or current administrator does not match the reviewed release")
        pending = record.get("pending_administrator")
        require(pending is None or public_account(pending), "invalid proposed successor")
        if parameter is None:
            require(record.get("paused") is None and pending is None,
                    "immutable gate has no live registry handover")
            continue
        require(type(record.get("paused")) is bool, "missing registry pause state")
        require(pending in (None, successor),
                "another administrator nomination is pending; manual review required")
        common = [
            "stellar", "contract", "invoke", "--id", cid, "--network", "testnet",
            "--source-account"
        ]
        if pending is None:
            commands.append({
                "contract": name, "phase": "propose", "signer": "old locally-authorized administrator",
                "command": common + ["operator_local", "--", "propose_admin", "--"+parameter, successor]
            })
        commands.append({
            "contract": name, "phase": "accept", "signer": "separately-authorized, recoverable successor",
            "command": common + ["NEW_RECOVERABLE_IDENTITY", "--", "accept_admin"]
        })
    return {
        "network": "testnet", "source_observation_ledger": ledger,
        "source_observation_closed_at_unix": close,
        "original_governance_admin": RELEASE_ADMIN,
        "proposed_registry_admin": successor,
        "gate_admin_is_immutable": True,
        "commands_for_review_only": commands,
        "signed": False, "broadcast": False, "safe_to_execute_automatically": False,
        "financial_execution_enabled": False,
        "checks_required_before_manual_actions": [
            "independently refresh current live Testnet executable hashes, administrator and pending nominations",
            "verify the new wallet can actually sign and its recovery backup works; do not expose secrets",
            "verify the old operator_local identity still signs on the original PC",
            "inspect the exact on-chain Soroban methods and authorization requirements with Stellar CLI",
            "manually authorize each proposal and accept with a DIFFERENT verified successor signer",
            "read current on-chain admin and pending state after every change; keep original credentials until confirmed",
            "GovernanceGate remains immutable and retains its original admin record"
        ],
    }

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--observation", required=True, type=Path,
                        help="Locally saved JSON response from GET /v1/governance/admin")
    parser.add_argument("--successor", required=True,
                        help="New checksummed public G address (never a seed)")
    args = parser.parse_args()
    try:
        raw = args.observation.read_bytes()
        require(100 < len(raw) <= 16_384, "observation outside reviewed byte bounds")
        result = plan(json.loads(raw), args.successor)
    except (OSError, ValueError, TypeError, json.JSONDecodeError) as err:
        parser.exit(1, f"Refused offline recovery proposal: {err}\n")
    print(json.dumps(result, indent=2, sort_keys=True))

if __name__ == "__main__":
    main()
