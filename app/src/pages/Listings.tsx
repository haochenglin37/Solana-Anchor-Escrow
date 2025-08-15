import { useEffect, useState } from 'react';
import { useAnchor } from '../lib/anchorClient';
import ListingCard from '../components/ListingCard';

export default function Listings() {
  const { program } = useAnchor();
  const [listings, setListings] = useState<any[]>([]);

  useEffect(() => {
    const load = async () => {
      if (!program) return;
      const all = await program.account.listing.all();
      setListings(all.filter((l) => l.account.status === 0));
    };
    load();
  }, [program]);

  return (
    <div>
      <h2>Open Listings</h2>
      {listings.map((l) => (
        <ListingCard key={l.publicKey.toBase58()} listing={l} />
      ))}
    </div>
  );
}
