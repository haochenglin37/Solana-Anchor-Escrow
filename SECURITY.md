# Security

## Threat Model
- Malicious buyer/seller attempting double spend or unauthorized withdrawals.
- CPI into untrusted programs.

## Permissions
| Instruction    | Who can call                                     |
|----------------|--------------------------------------------------|
| `createListing`| Seller (signer)                                  |
| `buy`          | Buyer (signer) while listing `Open`              |
| `cancel`       | Seller or anyone after `expiry`                  |
| `withdrawFee`  | Admin only (demo key hard coded)                 |

## Upgrade Authority
The program upgrade authority is a single key for demo purposes.  
In production, a multisig should own upgrades.

## Reentrancy & Replay
Solana transaction model executes atomically; however, we:
- Update listing state **before** transfers on `buy`.
- Reject any instruction if `status != Open`.

## CPI Allowlist
Only CPI to SPL Token & Associated Token program.  
Additional CPI should be explicitly whitelisted.

## Integer Safety
All arithmetic uses `checked_*` operations with early overflow checks.  
Inputs validated for non-zero price/quantity.
