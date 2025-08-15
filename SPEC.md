# Specification

## Accounts
### Listing
Seeds: `["listing", seller, mint, nonce]`  
Fields: seller, mint, price, quantity, expiry, status, bump, nonce.

### FeeVault
Seeds: `["fee_vault"]` – SPL token account to hold platform fees.

### Escrow ATA
Associated token account for listing PDA + mint.

## Status Machine
```
Open -> Settled (on `buy`)
Open -> Canceled (on `cancel`)
```
No further transitions allowed.

## Instructions
### createListing(price, quantity, expiry, nonce)
- seller signs.
- Transfers `quantity` of `mint` to escrow ATA.
- Emits `ListingCreated`.

### buy()
- buyer signs.
- Checks `status == Open`; sets to Settled.
- Transfers `price` from buyer to seller minus fee.
- Transfers fee to FeeVault.
- Releases escrowed tokens to buyer.
- Emits `Purchased`.

### cancel()
- seller or anyone after expiry.
- Status must be Open.
- Releases escrow to seller.
- Emits `Canceled`.

### withdrawFee(amount)
- admin signer.
- Moves `amount` from FeeVault to admin account.
- Emits `FeeWithdrawn`.

## Events
- `ListingCreated { seller, listing, mint, price, quantity }`
- `Purchased { listing, buyer, price }`
- `Canceled { listing, seller }`
- `FeeWithdrawn { admin, amount }`

## Errors
- `InvalidState`
- `Unauthorized`
- `Overflow`
- `InvalidQuantity`
- `InvalidPrice`
