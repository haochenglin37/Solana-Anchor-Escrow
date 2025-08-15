import { useState } from 'react';
import { useAnchor } from '../lib/anchorClient';
import { BN } from '@coral-xyz/anchor';

export default function CreateListingForm() {
  const { program, wallet } = useAnchor();
  const [price, setPrice] = useState('');
  const [qty, setQty] = useState('');
  const [expiry, setExpiry] = useState('');

  const submit = async () => {
    if (!program || !wallet) return;
    const nonce = Date.now();
    await program.methods
      .createListing(new BN(price), new BN(qty), new BN(expiry), new BN(nonce))
      .accounts({
        seller: wallet.publicKey,
      })
      .rpc();
  };

  return (
    <div>
      <h3>Create Listing</h3>
      <input placeholder="Price" value={price} onChange={(e) => setPrice(e.target.value)} />
      <input placeholder="Quantity" value={qty} onChange={(e) => setQty(e.target.value)} />
      <input placeholder="Expiry (unix)" value={expiry} onChange={(e) => setExpiry(e.target.value)} />
      <button onClick={submit}>Submit</button>
    </div>
  );
}
