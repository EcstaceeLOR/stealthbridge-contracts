# Recover the Testnet administrator before changing your PC

**This is an offline review workflow, not a signing, wallet import, or
automatic governance execution tool.** Only a current authorized signer
can nominate a new registry admin. Creating a new CLI alias with the
same name on another computer DOES NOT recreate the original key.

Current recorded deployment operator:
\`GC3YRVLB7IN7SNUYVWFUC6OD3QWJXCUZ4HOBNIPVGY7BZDVOYMDNX5TN\`.

## Safe handover checklist

1. Set up a **new securely recoverable** account, preferably protected by a
   hardware wallet or an explicitly reviewed multisig policy. Privately verify
   that you can restore and sign with the new wallet. Never send a seed here.
2. While the old computer is still operational, make sure
   \`stellar keys address operator_local\` returns the currently authorized
   on-chain G-address and that the secure-store key is usable.
3. From a TRUSTED **preview** API environment, review the current
   \`GET /v1/governance/code\` and \`GET /v1/governance/admin\` responses.
   The legacy not-deployed v1 manifest is not a current-state source.
4. Save the validated public JSON response from \`/v1/governance/admin\`
   **without credentials**, then run:
   \`\`\`bash
   python3 scripts/plan_governance_recovery.py \
     --observation observation.json \
     --successor REPLACE_WITH_CHECKSUM_VALID_NEW_G_ADDRESS
   \`\`\`
5. The planner never contacts a chain or wallet, cannot attest to the
   freshness of saved JSON, cannot tell whether either party can sign,
   and **never executes** the displayed CLI commands. Independently
   check current live RPC state immediately before each manual action.
6. The original admin proposes on **CorridorRegistry** with \`--new_admin\`
   and on **PolicyRegistry** with \`--successor\`. A different
   user-controlled signing identity for the **nominated new administrator**
   runs \`accept_admin\` for each registry. Each phase is an actual Testnet
   governance transaction and must be separately inspected/authorized.
7. Check \`get_admin\`/\`admin\` and \`pending_admin\` on both live registries
   after completion. Keep the original signer available until the results
   match the intended handover. Never infer ownership from a public G-string.
8. **GovernanceGate's constructor administrator is immutable.** Its
   \`get_admin\` may still return the original deployer after registry
   succession. GovernanceGate has no privileged post-deployment operations,
   but a new gate deployment would need a separate reviewed release.
9. The fee vault is a distinct, undeployed prototype. It has its OWN
   administrator and successor flow. Registry migrations do not
   automatically redirect claims from a future fee vault.

For Mainnet, establish multisig/hardware signing, tested backups and an
independent legal/security review first. No customer payments, transfer
execution or fee collection is authorized by this recovery planner.
