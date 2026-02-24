/**
 * Property-based tests using fast-check.
 * Run: npm run test
 *
 * Note: These tests validate pure logic invariants. For full property-based
 * testing of Anchor programs with randomized transactions, consider using
 * Trident framework or similar tools.
 */
import * as fc from "fast-check";
import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Vault } from "../target/types/vault";
import { expect } from "chai";
import {
  createTestMint,
  fundAta,
  deriveVaultPDA,
  getTokenBalance,
} from "./helpers";
import { Keypair } from "@solana/web3.js";
import { getAssociatedTokenAddress } from "@solana/spl-token";

// Pure logic extracted from on-chain state helpers — tested without RPC
function checkedAdd(a: number, b: number): number | null {
  const r = a + b;
  return r > Number.MAX_SAFE_INTEGER ? null : r;
}

function dailyLimitCheck(
  withdrawn: number,
  limit: number,
  amount: number
): boolean {
  if (amount === 0) return false;
  if (limit === 0) return true;
  const next = withdrawn + amount;
  return next <= limit;
}

describe("vault property tests", () => {
  // ─── Pure logic tests (no RPC) ────────────────────────────────────────────
  it("checkedAdd never returns more than sum of inputs (no overflow)", () => {
    fc.assert(
      fc.property(
        fc.nat({ max: 1e15 }),
        fc.nat({ max: 1e15 }),
        (a, b) => {
          const r = checkedAdd(a, b);
          return r === null || r === a + b;
        }
      )
    );
  });

  it("dailyLimitCheck is monotone — once exceeded stays exceeded", () => {
    fc.assert(
      fc.property(
        fc.nat({ max: 1e9 }),
        fc.nat({ max: 1e9 }),
        fc.nat({ max: 1e9 }),
        (withdrawn, limit, amount) => {
          const first = dailyLimitCheck(withdrawn, limit, amount);
          const second = dailyLimitCheck(withdrawn + amount, limit, amount);
          // if first fails, second should also fail (monotone)
          if (!first) return !second || limit === 0;
          return true;
        }
      )
    );
  });

  it("zero amount is always rejected", () => {
    fc.assert(
      fc.property(fc.nat({ max: 1e9 }), (limit) => {
        return !dailyLimitCheck(0, limit, 0);
      })
    );
  });

  // ─── On-chain property tests (with RPC) ──────────────────────────────────
  describe("on-chain invariants", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.Vault as Program<Vault>;
    const conn = provider.connection;
    const payer = (provider.wallet as anchor.Wallet).payer;

    it("property: deposit-withdraw roundtrip preserves balance", async () => {
      // Test with multiple random amounts
      await fc.assert(
        fc.asyncProperty(
          fc.nat({ max: 500_000 }).filter((n) => n > 0),
          async (amount) => {
            const owner = Keypair.generate();
            await conn.requestAirdrop(owner.publicKey, 2e9);
            await new Promise((r) => setTimeout(r, 500));

            const mint = await createTestMint(conn, payer);
            const ownerAta = await fundAta(
              conn,
              payer,
              mint,
              owner.publicKey,
              1_000_000
            );

            const [vaultState] = deriveVaultPDA(owner.publicKey, mint);
            const vaultAta = await getAssociatedTokenAddress(
              mint,
              vaultState,
              true
            );

            await program.methods
              .initialize(new BN(1_000_000), new BN(10_000_000))
              .accounts({
                owner: owner.publicKey,
                mint,
                vaultState,
                vaultAta,
              } as any)
              .signers([owner])
              .rpc();

            // Deposit
            const balanceBefore = Number(await getTokenBalance(conn, ownerAta));
            await program.methods
              .deposit(new BN(amount))
              .accounts({
                owner: owner.publicKey,
                mint,
                vaultState,
                ownerAta,
                vaultAta,
              } as any)
              .signers([owner])
              .rpc();

            // Withdraw same amount
            await program.methods
              .withdraw(new BN(amount))
              .accounts({
                owner: owner.publicKey,
                mint,
                vaultState,
                vaultAta,
                ownerAta,
              } as any)
              .signers([owner])
              .rpc();

            const balanceAfter = Number(await getTokenBalance(conn, ownerAta));

            // Invariant: roundtrip should preserve balance (minus dust/rounding)
            return Math.abs(balanceAfter - balanceBefore) < 2;
          }
        ),
        { numRuns: 3 } // Limited runs to avoid RPC rate limits
      );
    });

    it("property: vault total_deposited is monotonically increasing", async () => {
      await fc.assert(
        fc.asyncProperty(
          fc.array(fc.nat({ max: 100_000 }).filter((n) => n > 0), {
            minLength: 2,
            maxLength: 5,
          }),
          async (amounts) => {
            const owner = Keypair.generate();
            await conn.requestAirdrop(owner.publicKey, 2e9);
            await new Promise((r) => setTimeout(r, 500));

            const mint = await createTestMint(conn, payer);
            const ownerAta = await fundAta(
              conn,
              payer,
              mint,
              owner.publicKey,
              10_000_000
            );

            const [vaultState] = deriveVaultPDA(owner.publicKey, mint);
            const vaultAta = await getAssociatedTokenAddress(
              mint,
              vaultState,
              true
            );

            await program.methods
              .initialize(new BN(1_000_000), new BN(10_000_000))
              .accounts({
                owner: owner.publicKey,
                mint,
                vaultState,
                vaultAta,
              } as any)
              .signers([owner])
              .rpc();

            let prevTotal = 0;
            for (const amt of amounts) {
              await program.methods
                .deposit(new BN(amt))
                .accounts({
                  owner: owner.publicKey,
                  mint,
                  vaultState,
                  ownerAta,
                  vaultAta,
                } as any)
                .signers([owner])
                .rpc();

              const state = await program.account.vaultState.fetch(vaultState);
              const currentTotal = state.totalDeposited.toNumber();

              // Invariant: total_deposited never decreases
              if (currentTotal < prevTotal) return false;
              prevTotal = currentTotal;
            }

            return true;
          }
        ),
        { numRuns: 2 }
      );
    });
  });
});
