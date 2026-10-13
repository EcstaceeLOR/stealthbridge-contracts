import copy
import json
import sys
import unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"scripts"))
from verify_upstream_source_lock import validate_manifest,verify_bytes,hash_git_blob
DOC=json.loads((ROOT/"integrations/upstream-source-lock.v1.json").read_text())

class SourceProvenanceNegativeTests(unittest.TestCase):
 def test_no_go_manifest_requires_six_reviewed_public_blobs(self):
    self.assertEqual(len(validate_manifest(DOC)),6)
 def test_small_git_blob_sha1_matches_canonical_known_vector(self):
    self.assertEqual(hash_git_blob(b"hello"),"b6fc4c620b67d95f953a5c1c1230aaab5db5a1b0")
    entry={"gitBlobSha1":"b6fc4c620b67d95f953a5c1c1230aaab5db5a1b0","path":"sample"}
    self.assertEqual(verify_bytes(entry,b"hello"),entry["gitBlobSha1"])
    with self.assertRaisesRegex(ValueError,"bytes differ"):
      verify_bytes(entry,b"hello altered")
 def test_can_never_set_verified_or_enable_payments_from_source_hash(self):
    for key,value in [("independentlyReproduced",True),("proofVerified",True),
       ("paymentExecutionEnabled",True),("verificationStatus","verified")]:
      r=copy.deepcopy(DOC);r[key]=value
      with self.subTest(key=key),self.assertRaisesRegex(ValueError,"forged"):
        validate_manifest(r)
 def test_wrong_commit_blob_or_circuit_path_is_rejected(self):
    for key,value in [("commit","f"*40),("gitBlobSha1","0"*40),
      ("path","packages/confidential/circuits/transfer/src/backdoor.nr"),
      ("repository","unknown/malicious")]:
      r=copy.deepcopy(DOC);r["sources"][2][key]=value
      with self.subTest(key=key),self.assertRaisesRegex(ValueError,"does not match"):
        validate_manifest(r)
 def test_merging_rails_or_omitting_nullifier_circuit_source_fails(self):
    r=copy.deepcopy(DOC);r["sources"][3]["rail"]="confidential-token"
    with self.assertRaisesRegex(ValueError,"does not match"):
      validate_manifest(r)
    r=copy.deepcopy(DOC);r["sources"].pop()
    with self.assertRaisesRegex(ValueError,"exactly six"):
      validate_manifest(r)
 def test_secret_and_unbounded_data_rejected(self):
    entry={"gitBlobSha1":"0"*40,"path":"test"}
    for body in [b"",b"x"*256001]:
      with self.assertRaisesRegex(ValueError,"unbounded"):
        verify_bytes(entry,body)
if __name__=="__main__":unittest.main()
