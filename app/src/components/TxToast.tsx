import { useEffect, useState } from 'react';

export default function TxToast({ signature }: { signature?: string }) {
  const [url, setUrl] = useState<string>();

  useEffect(() => {
    if (signature) {
      setUrl(`https://solscan.io/tx/${signature}?cluster=devnet`);
    }
  }, [signature]);

  if (!signature) return null;
  return (
    <div style={{ position: 'fixed', bottom: 10, right: 10, background: '#eee', padding: '0.5rem' }}>
      <a href={url} target="_blank" rel="noreferrer">
        View transaction
      </a>
    </div>
  );
}
