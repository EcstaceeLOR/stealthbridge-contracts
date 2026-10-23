#!/usr/bin/env python3
"""Read-only independent *receipt existence* check against Stellar Testnet RPC.

This is NOT a ZK verifier. SUCCESS only proves the queried transaction was
recorded with that status, not that a payment was confidential or authorized.
Never accepts private keys, decrypted notes, proof witnesses or customer data.
"""
import argparse
import json
import re
from urllib.parse import urlparse
from urllib.request import Request, urlopen

TESTNET_PASSPHRASE = "Test SDF Network ; September 2015"
HASH_PATTERN = re.compile(r"[0-9a-fA-F]{64}\Z")


class ReceiptError(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise ReceiptError(message)


def rpc_call(url, method, params):
    payload = json.dumps({
        "jsonrpc": "2.0", "id": 1, "method": method, "params": params
    }).encode("utf-8")
    request = Request(url, data=payload, headers={
        "Content-Type": "application/json",
    })
    with urlopen(request, timeout=15) as response:
        body = response.read(262145)
    require(len(body) <= 262144, "oversized RPC response")
    try:
        data = json.loads(body)
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise ReceiptError("invalid RPC JSON") from exc
    require(type(data) is dict and "error" not in data
            and type(data.get("result")) is dict, "RPC error or missing result")
    return data["result"]


def inspect_receipt(tx_hash, expected_status, rpc_url, rpc=rpc_call, confirm_rpc_url=None):
    """Check actual public Testnet receipt status, never cryptographic privacy."""
    require(type(tx_hash) is str and HASH_PATTERN.fullmatch(tx_hash),
            "expected a 64-hex transaction hash")
    require(expected_status in ("SUCCESS", "FAILED"), "expected SUCCESS or FAILED")
    parsed = urlparse(rpc_url)
    require(parsed.scheme == "https" and parsed.hostname
            and parsed.username is None and parsed.password is None
            and parsed.fragment == "", "RPC must be HTTPS without embedded credentials")
    network = rpc(rpc_url, "getNetwork", {})
    require(network.get("passphrase") == TESTNET_PASSPHRASE,
            "RPC is not Stellar Testnet")
    receipt = rpc(rpc_url, "getTransaction", {"hash": tx_hash.lower()})
    require(receipt.get("hash", "").lower() == tx_hash.lower(),
            "RPC returned different or missing transaction hash")
    require(receipt.get("status") == expected_status,
            "transaction status differs from expected outcome")
    ledger = receipt.get("ledger")
    require(type(ledger) is int and ledger > 0, "missing positive ledger number")
    # Requiring independent RPC endpoints reduces reliance on one provider,
    # but matching receipts do NOT independently validate ZK proofs.
    confirmation = False
    if confirm_rpc_url is not None:
        other = urlparse(confirm_rpc_url)
        require(other.scheme == "https" and other.hostname
                and other.username is None and other.password is None
                and other.fragment == "" and confirm_rpc_url != rpc_url,
                "confirmation RPC must be separate HTTPS endpoint")
        second_network = rpc(confirm_rpc_url, "getNetwork", {})
        require(second_network.get("passphrase") == TESTNET_PASSPHRASE,
                "confirmation RPC is not Stellar Testnet")
        second = rpc(confirm_rpc_url, "getTransaction", {"hash": tx_hash.lower()})
        require(second.get("hash", "").lower() == tx_hash.lower()
                and second.get("status") == expected_status
                and type(second.get("ledger")) is int
                and second["ledger"] == ledger,
                "independent Testnet receipts disagree")
        confirmation = True
    # Do not echo arbitrary RPC fields: envelope XDR and return values may be
    # sensitive, and no proof/ledger observer analysis has been carried out.
    return {
        "network": "testnet",
        "hash": tx_hash.lower(),
        "ledger": ledger,
        "status": expected_status,
        "receiptChecked": True,
        "confirmedBySecondRpc": confirmation,
        "privacyProven": False,
        "zkProofVerified": False,
        "walletLinkabilityReviewed": False,
        "paymentEnabled": False,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--hash", required=True, help="Public Testnet transaction hash")
    parser.add_argument("--expected-status", choices=("SUCCESS", "FAILED"), required=True)
    parser.add_argument("--rpc-url", required=True, help="First HTTPS Testnet RPC endpoint")
    parser.add_argument("--confirm-rpc-url", required=True,
                        help="Different HTTPS Testnet RPC endpoint for independent receipt")
    args = parser.parse_args()
    print(json.dumps(inspect_receipt(
        args.hash, args.expected_status, args.rpc_url,
        confirm_rpc_url=args.confirm_rpc_url
    ), sort_keys=True, indent=2))


if __name__ == "__main__":
    main()
