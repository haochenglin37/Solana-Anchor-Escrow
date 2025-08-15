import { AnchorProvider, Program, Idl } from '@coral-xyz/anchor';
import { Connection, PublicKey } from '@solana/web3.js';
import { useWallet } from '@solana/wallet-adapter-react';
import idl from '../../../IDL/escrow_market.json';

const programId = new PublicKey(import.meta.env.VITE_PROGRAM_ID || 'Escrow1111111111111111111111111111111111111');

export const useAnchor = () => {
  const wallet = useWallet();
  const connection = new Connection(import.meta.env.VITE_RPC_URL || 'https://api.devnet.solana.com');
  const provider = wallet.publicKey ? new AnchorProvider(connection, wallet, {}) : undefined;
  const program = provider ? new Program(idl as Idl, programId, provider) : undefined;
  return { program, wallet };
};
