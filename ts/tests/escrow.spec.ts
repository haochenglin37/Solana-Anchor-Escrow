import * as anchor from '@coral-xyz/anchor';
import { Program, BN } from '@coral-xyz/anchor';
import { EscrowMarket } from '../../target/types/escrow_market';
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID
} from '@solana/spl-token';
import { listingPda, feeVaultPda } from '../sdk';

describe('escrow_market', () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.EscrowMarket as Program<EscrowMarket>;

  let mint: anchor.web3.PublicKey;
  let sellerToken: anchor.web3.PublicKey;
  let buyerToken: anchor.web3.PublicKey;
  let seller = anchor.web3.Keypair.generate();
  let buyer = anchor.web3.Keypair.generate();
  const price = new BN(100);
  const qty = new BN(50);
  const nonce = new BN(1);
  const expiry = new BN(Math.floor(Date.now() / 1000) + 3600);

  it('setup accounts', async () => {
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(seller.publicKey, 1e9)
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(buyer.publicKey, 1e9)
    );
    mint = await createMint(provider.connection, seller, seller.publicKey, null, 0);
    sellerToken = (await getOrCreateAssociatedTokenAccount(provider.connection, seller, mint, seller.publicKey)).address;
    buyerToken = (await getOrCreateAssociatedTokenAccount(provider.connection, buyer, mint, buyer.publicKey)).address;
    await mintTo(provider.connection, seller, mint, sellerToken, seller, 200);
    await mintTo(provider.connection, seller, mint, buyerToken, seller, 200);
  });

  it('create listing', async () => {
    await program.methods
      .createListing(price, qty, expiry, nonce)
      .accounts({
        seller: seller.publicKey,
        mint,
        sellerTokenAccount: sellerToken
      })
      .signers([seller])
      .rpc();

    const [listing] = listingPda(seller.publicKey, mint, nonce);
    const l = await program.account.listing.fetch(listing);
    anchor.assert(l.price.eq(price));
  });

  it('buy listing', async () => {
    const [listing] = listingPda(seller.publicKey, mint, nonce);
    const feeVault = (await feeVaultPda())[0];
    await program.methods
      .buy()
      .accounts({
        buyer: buyer.publicKey,
        listing,
        buyerPaymentAccount: buyerToken,
        buyerReceiveAccount: buyerToken,
        sellerReceivingAccount: sellerToken,
        escrowTokenAccount: (await anchor.utils.token.associatedAddress({ mint, owner: listing })),
        feeVault,
        feeVaultAuthority: feeVault,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([buyer])
      .rpc();
    const l = await program.account.listing.fetch(listing);
    anchor.assert.equal(l.status, 1);
  });

  it('double buy fails', async () => {
    const [listing] = listingPda(seller.publicKey, mint, nonce);
    try {
      await program.methods
        .buy()
        .accounts({
          buyer: buyer.publicKey,
          listing,
          buyerPaymentAccount: buyerToken,
          buyerReceiveAccount: buyerToken,
          sellerReceivingAccount: sellerToken,
          escrowTokenAccount: (await anchor.utils.token.associatedAddress({ mint, owner: listing })),
          feeVault: (await feeVaultPda())[0],
          feeVaultAuthority: (await feeVaultPda())[0],
          tokenProgram: TOKEN_PROGRAM_ID
        })
        .signers([buyer])
        .rpc();
      throw new Error('should fail');
    } catch (err) {
      // expected
    }
  });

  it('withdraw fee', async () => {
    const feeVault = (await feeVaultPda())[0];
    const adminToken = buyerToken; // demo
    await program.methods
      .withdrawFee(new BN(1))
      .accounts({
        admin: provider.wallet.publicKey,
        adminFeeAccount: adminToken,
        feeVault,
        feeVaultAuthority: feeVault,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .rpc();
  });
});
