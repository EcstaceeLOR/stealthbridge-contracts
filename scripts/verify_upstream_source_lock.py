#!/usr/bin/env python3
"""Verify pinned PUBLIC upstream Git blobs, never ZK proofs or live payments."""
import hashlib
import json
import re
from pathlib import Path
from urllib.request import Request, urlopen

ROOT=Path(__file__).resolve().parents[1]
LOCK=ROOT/"integrations/upstream-source-lock.v1.json"
REFS={
 "confidential-token":("OpenZeppelin/stellar-contracts",
  "6b2f03402b6b98fad6b4247b5e15dce91a0b205d",{
   "packages/confidential/README.md":"d14f1c94c9642570717c9904c87822d0e35f44d3",
   "packages/confidential/docs/protocol/security.md":"ce2e211db4644fefc6370d2c60a048333f22cdf6",
   "packages/confidential/circuits/transfer/src/main.nr":"b652748a68a7eb7757d79f95e7eb4af5675d94c0",
  }),
 "private-payments":("NethermindEth/stellar-private-payments",
  "0471e6be8161c2013cfc8387bd7b72b70e480cf5",{
   "README.md":"4592d032decf605072be4063c60f1e81e0d2d9cf",
   "circuits/src/policy_tx_2_2.circom":"bb6b47b867ca0928eeb8f93d63465f778f1a60a8",
   "circuits/src/core/merkle.rs":"0599bdf12cb71484bbdf75ca0cc6b032f39905bb",
  })
}
def require(condition,message):
    if not condition:raise ValueError(message)

def validate_manifest(record):
    require(type(record) is dict and set(record)=={
      "schemaVersion","network","verificationStatus","independentlyReproduced",
      "proofVerified","paymentExecutionEnabled","notes","sources"},
      "unreviewed source-only evidence metadata")
    require(type(record["schemaVersion"]) is int and record["schemaVersion"]==1 and
      record["network"]=="testnet" and
      record["verificationStatus"]=="source-only-not-verified" and
      record["independentlyReproduced"] is False and
      record["proofVerified"] is False and record["paymentExecutionEnabled"] is False,
      "forged testnet privacy verification or financial execution claim")
    require(type(record["notes"]) is str and len(record["notes"])>50,
      "missing proof/transfer limitation")
    entries=record["sources"]
    require(type(entries) is list and len(entries)==6,"exactly six pinned upstream source files required")
    expected=[(rail,repo,commit,path,sha)
      for rail,(repo,commit,paths) in REFS.items() for path,sha in paths.items()]
    for entry,correct in zip(entries,expected):
        rail,repo,commit,path,sha=correct
        require(type(entry) is dict and set(entry)=={
          "rail","repository","commit","path","gitBlobSha1"},
          "unsupported source evidence fields")
        require(tuple(entry.get(k) for k in (
          "rail","repository","commit","path","gitBlobSha1"))==correct,
          "upstream path, circuit or Git object ref does not match reviewed source")
        require(re.fullmatch(r"[a-f0-9]{40}",sha) is not None,
          "malformed reviewed Git object hash")
    return entries

def hash_git_blob(raw):
    # Git hashes the exact bytes of a blob with the canonical length header.
    return hashlib.sha1(b"blob "+str(len(raw)).encode("ascii")+b"\0"+raw).hexdigest()

def verify_bytes(entry,body):
    require(type(body) is bytes and 0<len(body)<=256_000,"unbounded upstream source blob")
    sha=hash_git_blob(body)
    require(sha==entry["gitBlobSha1"],f"pinned upstream source bytes differ at {entry['path']}")
    return sha

def verify_live(record):
    entries=validate_manifest(record)
    for entry in entries:
        url="https://raw.githubusercontent.com/{}/{}/{}".format(
          entry["repository"],entry["commit"],entry["path"])
        request=Request(url,headers={
          "User-Agent":"StealthBridge-source-provenance-gate/1.0",
          "Accept":"text/plain",
        })
        with urlopen(request,timeout=12) as response:
            require(response.status==200 and response.url==url,
                "upstream source redirected or unavailable")
            body=response.read(256_001)
        sha=verify_bytes(entry,body)
        print(f"PASS pinned {entry['rail']} {entry['path']} ({sha[:12]})")
    return len(entries)

if __name__=="__main__":
    record=json.loads(LOCK.read_text())
    n=verify_live(record)
    print(f"PASS {n} source blobs; independent ZK proof/Testnet wallet verification still NOT DONE")
