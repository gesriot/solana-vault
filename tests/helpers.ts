import {
  createMint,
  createAssociatedTokenAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { Keypair, PublicKey, Connection } from "@solana/web3.js";

export const PROGRAM_ID = new PublicKey(
  "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"
);

export async function createTestMint(
  connection: Connection,
  payer: Keypair
): Promise<PublicKey> {
  return createMint(connection, payer, payer.publicKey, null, 6);
}

export async function fundAta(
  connection: Connection,
  payer: Keypair,
  mint: PublicKey,
  owner: PublicKey,
  amount: number
): Promise<PublicKey> {
  const ata = await createAssociatedTokenAccount(connection, payer, mint, owner);
  await mintTo(connection, payer, mint, ata, payer, amount);
  return ata;
}

export function deriveVaultPDA(
  owner: PublicKey,
  mint: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), owner.toBuffer(), mint.toBuffer()],
    PROGRAM_ID
  );
}

export function deriveDelegatePDA(
  vault: PublicKey,
  delegate: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("delegate"), vault.toBuffer(), delegate.toBuffer()],
    PROGRAM_ID
  );
}

export async function getTokenBalance(
  connection: Connection,
  ata: PublicKey
): Promise<bigint> {
  const account = await getAccount(connection, ata);
  return account.amount;
}
