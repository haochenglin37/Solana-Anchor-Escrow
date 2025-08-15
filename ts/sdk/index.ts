import { PublicKey, TransactionInstruction, SystemProgram } from '@solana/web3.js';
import { BN, Program } from '@coral-xyz/anchor';
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { EscrowMarket } from '../../target/types/escrow_market';

export const PROGRAM_ID = new PublicKey(process.env.PROGRAM_ID ?? 'Escrow1111111111111111111111111111111111111');

export const listingPda = (seller: PublicKey, mint: PublicKey, nonce: BN) =>
  PublicKey.findProgramAddressSync(
    [Buffer.from('listing'), seller.toBuffer(), mint.toBuffer(), nonce.toArrayLike(Buffer, 'le', 8)],
    PROGRAM_ID
  );

export const feeVaultPda = () => PublicKey.findProgramAddressSync([Buffer.from('fee_vault')], PROGRAM_ID);

export const escrowAta = (listing: PublicKey, mint: PublicKey) =>
  PublicKey.findProgramAddressSync([listing.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()], ASSOCIATED_TOKEN_PROGRAM_ID);

export const createListingIx = async (
  program: Program<EscrowMarket>,
  seller: PublicKey,
  mint: PublicKey,
  sellerToken: PublicKey,
  price: BN,
  qty: BN,
  expiry: BN,
  nonce: BN
): Promise<TransactionInstruction> => {
  const [listing] = listingPda(seller, mint, nonce);
  const [escrow] = escrowAta(listing, mint);
  return program.methods.createListing(price, qty, expiry, nonce).accounts({
    seller,
    listing,
    mint,
    sellerTokenAccount: sellerToken,
    escrowTokenAccount: escrow,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId
  }).instruction();
};

export const buyIx = async (
  program: Program<EscrowMarket>,
  buyer: PublicKey,
  listing: PublicKey,
  buyerPay: PublicKey,
  buyerReceive: PublicKey,
  sellerReceive: PublicKey
): Promise<TransactionInstruction> => {
  const [feeVault] = feeVaultPda();
  const [feeVaultAuthority] = feeVaultPda();
  return program.methods.buy().accounts({
    buyer,
    listing,
    buyerPaymentAccount: buyerPay,
    buyerReceiveAccount: buyerReceive,
    sellerReceivingAccount: sellerReceive,
    escrowTokenAccount: (await escrowAta(listing, buyerReceive))[0],
    feeVault,
    feeVaultAuthority,
    tokenProgram: TOKEN_PROGRAM_ID
  }).instruction();
};
