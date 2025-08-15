import { useEffect, useState } from 'react';
import { useAnchor } from '../lib/anchorClient';
import CreateListingForm from '../components/CreateListingForm';
import ListingCard from '../components/ListingCard';

export default function MyListings() {
  const { program, wallet } = useAnchor();
  const [mine, setMine] = useState<any[]>([]);

  useEffect(() => {
    const load = async () => {
      if (!program || !wallet?.publicKey) return;
      const all = await program.account.listing.all([
        {
          memcmp: { offset: 8, bytes: wallet.publicKey.toBase58() }
        }
      ]);
      setMine(all);
    };
    load();
  }, [program, wallet]);

  return (
    <div>
      <CreateListingForm />
      <h2>My Listings</h2>
      {mine.map((l) => (
        <ListingCard key={l.publicKey.toBase58()} listing={l} />
      ))}
    </div>
  );
}
