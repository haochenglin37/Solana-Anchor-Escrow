# Solana Escrow Marketplace (Demo)

Minimal, production-style escrow marketplace built with Anchor.  
Users create listings for an SPL token, lock assets in escrow, and buyers purchase them in a single transaction with platform fees.

## Stack
- Anchor + Rust on-chain program
- React + Vite + TypeScript frontend
- SPL Token, Associated Token programs
- Wallet adapter (Phantom)
- Mocha/Anchor tests
- Linting via ESLint/Prettier, `cargo clippy`, `cargo-deny`

## Quickstart
```bash
# Build & test program
anchor build
anchor test

# Start frontend
cd app
npm install
npm run dev
```

## Env
Frontend expects:
```
VITE_RPC_URL=https://api.devnet.solana.com
VITE_PROGRAM_ID=Escrow1111111111111111111111111111111111111
VITE_MINT=<token mint>
```

## Deploy
```
anchor deploy
# update VITE_PROGRAM_ID in .env or vite config
```

## Notes
- Fee vault collects platform fees; admin withdraw using `withdrawFee`.
- Example only: no partial fills, assumes single mint, and demo admin key.
- Future: partial fills, NFT support, Token-2022, multisig admin.
