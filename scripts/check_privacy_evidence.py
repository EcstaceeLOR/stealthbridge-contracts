#!/usr/bin/env python3
"""Pre-execution privacy evidence release gate. Never accepts unverifiable claims."""
import json
import re
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
GATES={"cryptographicSoundness","independentAudit","crossWalletReproduction",
       "publicDisclosureReview","ledgerEvidence","issuerOrPoolAuthorization"}
SPECS={
 "confidential-token":{
  "repo":"OpenZeppelin/stellar-contracts",
  "ref":"6b2f03402b6b98fad6b4247b5e15dce91a0b205d",
  "proof":"UltraHonk",
  "privacy":"amount-confidential-parties-public",
  "cases":("ct-valid-testnet-transfer","ct-wrong-public-input",
           "ct-wrong-verification-key","ct-freeze-bypass",
           "ct-unauthorized-auditor","ct-public-edge"),
  "outcomes":("requires-cryptographic-reproduction",
              "proof-rejected-no-state-change",
              "proof-rejected-no-state-change",
              "authorization-rejected-no-state-change",
              "no-plaintext-disclosure",
              "underlying-amount-and-account-public"),
 },
 "private-payments":{
  "repo":"NethermindEth/stellar-private-payments",
  "ref":"0471e6be8161c2013cfc8387bd7b72b70e480cf5",
  "proof":"Groth16",
  "privacy":"counterparty-and-amount-shielded-with-public-edges",
  "cases":("spp-testnet-round-trip","spp-nullifier-replay",
           "spp-forged-membership","spp-bad-withdraw",
           "spp-external-edge","spp-client-state-loss"),
  "outcomes":("requires-cryptographic-reproduction",
              "proof-rejected-no-state-change",
              "proof-rejected-no-state-change",
              "proof-rejected-no-state-change",
              "external-amount-public",
              "no-fabricated-balance-or-success"),
 }
}

def require(condition,message):
    if not condition:raise ValueError(message)

def validate(record):
    require(type(record) is dict and set(record)=={
            "schemaVersion","network","networkPassphrase","verificationStatus",
            "paymentExecutionEnabled","requiredGates","rails","conclusion"},
            "unsupported privacy evidence record shape")
    require(type(record["schemaVersion"]) is int and record["schemaVersion"]==1 and
            record["network"]=="testnet" and
            record["networkPassphrase"]=="Test SDF Network ; September 2015",
            "unverified privacy network/schema")
    require(record["verificationStatus"]=="not-verified" and
            record["paymentExecutionEnabled"] is False and
            record["conclusion"]=="NO_GO",
            "unsupported private-payment execution or proof claim")
    gates=record["requiredGates"]
    require(type(gates) is dict and set(gates)==GATES and
            all(value is False for value in gates.values()),
            "evidence gate cannot be marked passed without independent verifier")
    require(type(record["rails"]) is list and len(record["rails"])==2,
            "both separate privacy tracks required")
    for row,(name,spec) in zip(record["rails"],SPECS.items()):
        require(type(row) is dict and set(row)=={
            "rail","upstreamRepo","upstreamCommit","proofSystem","privacyIntent",
            "sourceProtocol","vectors"} and row["rail"]==name and
            row["upstreamRepo"]==spec["repo"] and
            row["upstreamCommit"]==spec["ref"] and
            re.fullmatch(r"[a-f0-9]{40}",row["upstreamCommit"]) and
            row["proofSystem"]==spec["proof"] and
            row["privacyIntent"]==spec["privacy"],
            f"unreviewed or conflated {name} source/proof claim")
        expected_protocol=29 if name=="confidential-token" else None
        require(row["sourceProtocol"]==expected_protocol,
                "unreviewed protocol compatibility claim")
        vectors=row["vectors"]
        require(type(vectors) is list and len(vectors)==6,
                f"missing negative/positive {name} vectors")
        for vector,case,outcome in zip(vectors,spec["cases"],spec["outcomes"]):
            require(type(vector) is dict and set(vector)=={
                "id","inputClass","expected","evidence"} and
                vector["id"]==case and
                type(vector["inputClass"]) is str and
                len(vector["inputClass"])>10 and
                vector["expected"]==outcome and vector["evidence"] is None,
                f"unverified or altered privacy vector {case}")
    return 12

if __name__=="__main__":
    record=json.loads((ROOT/"integrations/privacy-evidence-vectors.v1.json").read_text())
    print(f"PASS: {validate(record)} static test specifications are pending; no proof or payout attested")
