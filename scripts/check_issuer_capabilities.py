#!/usr/bin/env python3
"""Source-only issuer capability gate. Cannot attest on-chain privacy."""
import json
import re
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
NAMES=("underlying_mint_burn","wrapper_deposit_withdraw",
       "amount_confidential_transfer","freeze_and_compliance_policy",
       "auditor_view","selective_disclosure","admin_rotation",
       "clawback","supply_invariants","fiat_redemption")
FIELDS={"schemaVersion","reviewDate","primitive","upstream","local",
        "capabilities","disclosures","decision"}
UPSTREAM={"repository","commit","workspaceVersion","sorobanSdk",
          "cliExample","protocolExample"}
LOCAL={"sdk","targetProtocol","manifestStatus","privacyVerified",
       "paymentExecutionEnabled","issuerAgreementVerified"}
DISCLOSURES={"counterpartyAddresses","privateTransferAmounts",
             "depositWithdrawAmounts","eventsTiming","issuerRedemption"}

def require(ok,why):
    if not ok:raise ValueError(why)

def validate(record):
    require(type(record) is dict and set(record)==FIELDS,
            "issuer capability record has missing or unknown fields")
    require(type(record["schemaVersion"]) is int and record["schemaVersion"]==1
            and record["primitive"]=="issuer-controlled-confidential-token"
            and record["decision"]=="NO_GO", "issuer capability is not approved")
    u=record["upstream"];v=record["local"]
    require(type(u) is dict and set(u)==UPSTREAM and
            u["repository"]=="OpenZeppelin/stellar-contracts" and
            re.fullmatch(r"[a-f0-9]{40}",u["commit"]) is not None and
            u["commit"]=="6b2f03402b6b98fad6b4247b5e15dce91a0b205d" and
            u["workspaceVersion"]=="0.7.1" and u["sorobanSdk"]=="28.0.0" and
            u["cliExample"]=="28.1.0" and type(u["protocolExample"]) is int and
            u["protocolExample"]==29, "unreviewed upstream toolchain")
    require(type(v) is dict and set(v)==LOCAL and v["sdk"]=="27.0.6" and
            type(v["targetProtocol"]) is int and v["targetProtocol"]==27 and
            v["manifestStatus"]=="not-deployed" and v["privacyVerified"] is False and
            v["paymentExecutionEnabled"] is False and
            v["issuerAgreementVerified"] is False, "unverified issuer capability claim")
    rows=record["capabilities"]
    require(type(rows) is list and len(rows)==len(NAMES),"incomplete issuer capability inventory")
    for row,name in zip(rows,NAMES):
        require(type(row) is dict and
                set(row)=={"name","upstream","stealthbridge","testnetReceipt","externalAudit"} and
                row["name"]==name and row["upstream"]=="documented-preview" and
                row["stealthbridge"]=="unverified" and
                row["testnetReceipt"] is None and row["externalAudit"] is None,
                f"unverified capability must remain fail closed: {name}")
    d=record["disclosures"]
    require(type(d) is dict and set(d)==DISCLOSURES and
            d["counterpartyAddresses"]=="public" and
            d["privateTransferAmounts"]=="intended-confidential-unverified" and
            d["depositWithdrawAmounts"]=="public" and
            d["eventsTiming"]=="public" and
            d["issuerRedemption"]=="off-chain-external",
            "privacy disclosure boundaries cannot be overstated")
    return len(rows)

if __name__=="__main__":
    count=validate(json.loads((ROOT/"integrations/confidential-tokens/capability-review.v1.json").read_text()))
    print(f"PASS: {count} issuer-related capabilities remain source-only, unverified and execution-disabled")
