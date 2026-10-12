#!/usr/bin/env python3
"""Validate public method inventory against the real checked-in Soroban sources.

This confirms names/signatures exist in source, NOT that any contract exists
on-chain or that simulating a read succeeds.
"""
import json,re
from pathlib import Path
metadata=json.loads(Path("integrations/public-soroban-interface.v1.json").read_text())
assert metadata["schemaVersion"]==1 and metadata["network"]=="testnet"
assert metadata["status"]=="source-interface-only"
assert set(metadata["contracts"])=={"corridor-registry","policy-registry","governance-gate"}
for name,contract in metadata["contracts"].items():
    source=Path(contract["source"]).read_text()
    methods=set(re.findall(r"pub fn ([a-z_]+)\s*\(",source))
    declared=set(contract["reads"])|set(contract["writes"])
    # Constructor and audited corridor upgrade lifecycle are intentionally
    # outside source-only read discovery; all other public functions must
    # appear in the reviewed ABI inventory. An unlisted new method is drift.
    expected_undisclosed = {
        "corridor-registry": {"__constructor", "propose_upgrade", "pending_upgrade",
                              "cancel_upgrade", "execute_upgrade"},
        "policy-registry": {"__constructor"},
        "governance-gate": {"__constructor"},
    }[name]
    assert methods-declared == expected_undisclosed, (
        name, "unlisted or missing public methods",
        sorted(methods-declared), sorted(expected_undisclosed))
    assert set(contract["reads"]).isdisjoint(set(contract["writes"])),name
    for method,info in contract["reads"].items():
        assert isinstance(info["args"],list) and len(info["args"]) <= {"corridor-registry":2,"policy-registry":3,"governance-gate":4}[name],name
        assert info["returns"] in {"Address","Option<Address>","bool","Option<PolicyRecord>","Result<Vec<bool>,GateError>"},method
        start=re.search(r"pub fn "+method+r"\s*\(",source)
        assert start is not None,method
        definition=source[start.start():start.start()+320]
        # Public governance calls must bind the exact revision/hash types.
        required={
            ("corridor-registry","is_enabled_with_digest"):["String","BytesN<32>"],
            ("policy-registry","is_effective_commitment"):["String","u32","BytesN<32>"],
            ("governance-gate","public_flags_allow_commitment"):["String","String","u32","BytesN<32>"],
            ("governance-gate","check_commitment_batch"):["Vec<GovernanceCheck>"],
        }
        if (name,method) in required:
            assert info["args"]==required[(name,method)] and info["returns"]==("Result<Vec<bool>,GateError>" if method=="check_commitment_batch" else "bool")
        for arg in info["args"]:
            assert arg in definition,(name,method,arg)
print("Soroban public method inventory matches all source contracts; no deployment asserted.")
