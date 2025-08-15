import { useAnchor } from '../lib/anchorClient';
import { PublicKey } from '@solana/web3.js';

interface Props {
  listing: any;
}

export default function ListingCard({ listing }: Props) {
  const { program, wallet } = useAnchor();

  const buy = async () => {
    if (!program || !wallet) return;
    await program.methods.buy().accounts({ buyer: wallet.publicKey, listing: listing.publicKey }).rpc();
  };

  const cancel = async () => {
    if (!program || !wallet) return;
    await program.methods.cancel().accounts({
      authority: wallet.publicKey,
      listing: listing.publicKey,
      seller: new PublicKey(listing.account.seller),
    }).rpc();
  };

  return (
    <div style={{ border: '1px solid gray', padding: '1rem', margin: '0.5rem' }}>
      <div>Seller: {listing.account.seller}</div>
      <div>Price: {listing.account.price.toString()}</div>
      <div>Quantity: {listing.account.quantity.toString()}</div>
      <div>Expiry: {listing.account.expiry.toString()}</div>
      <div>Status: {listing.account.status}</div>
      <button onClick={buy}>Buy</button>
      <button onClick={cancel}>Cancel</button>
    </div>
  );
}
