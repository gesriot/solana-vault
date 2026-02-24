import * as anchor from "@coral-xyz/anchor";
import { Program, BN }  from "@coral-xyz/anchor";
import { Vault }        from "../target/types/vault";
import {
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { Keypair } from "@solana/web3.js";
import { expect } from "chai";
import {
  createTestMint,
  fundAta,
  deriveVaultPDA,
  deriveDelegatePDA,
  getTokenBalance,
} from "./helpers";

describe("vault", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program  = anchor.workspace.Vault as Program<Vault>;
  const conn     = provider.connection;
  const payer    = (provider.wallet as anchor.Wallet).payer;

  let mint:       anchor.web3.PublicKey;
  let ownerAta:   anchor.web3.PublicKey;
  let vaultState: anchor.web3.PublicKey;
  let vaultAta:   anchor.web3.PublicKey;

  const MAX_DEPOSIT = 1_000_000;
  const DAILY_LIMIT = 5_000_000;

  // ─── setup ──────────────────────────────────────────────────────────────────
  before(async () => {
    mint     = await createTestMint(conn, payer);
    ownerAta = await fundAta(conn, payer, mint, payer.publicKey, 10_000_000);

    [vaultState] = deriveVaultPDA(payer.publicKey, mint);
    vaultAta     = await getAssociatedTokenAddress(mint, vaultState, true);
  });

  // ─── initialize ─────────────────────────────────────────────────────────────
  describe("initialize", () => {
    it("creates vault state with correct params", async () => {
      await program.methods
        .initialize(new BN(MAX_DEPOSIT), new BN(DAILY_LIMIT))
        .accounts({
          owner:       payer.publicKey,
          mint,
          vaultState,
          vaultAta,
        } as any)
        .rpc();

      const state = await program.account.vaultState.fetch(vaultState);
      expect(state.owner.toString()).to.equal(payer.publicKey.toString());
      expect(state.maxDeposit.toNumber()).to.equal(MAX_DEPOSIT);
      expect(state.dailyWithdrawLimit.toNumber()).to.equal(DAILY_LIMIT);
      expect(state.locked).to.be.false;
    });

    it("rejects double-init (account already exists)", async () => {
      try {
        await program.methods
          .initialize(new BN(MAX_DEPOSIT), new BN(DAILY_LIMIT))
          .accounts({ owner: payer.publicKey, mint, vaultState, vaultAta } as any)
          .rpc();
        expect.fail("should have thrown");
      } catch (e: any) {
        expect(e.message).to.match(/already in use/i);
      }
    });
  });

  // ─── deposit ─────────────────────────────────────────────────────────────────
  describe("deposit", () => {
    it("transfers tokens to vault ATA", async () => {
      const before = await getTokenBalance(conn, vaultAta);

      await program.methods
        .deposit(new BN(500_000))
        .accounts({ owner: payer.publicKey, mint, vaultState, ownerAta, vaultAta } as any)
        .rpc();

      const after = await getTokenBalance(conn, vaultAta);
      expect(Number(after - before)).to.equal(500_000);
    });

    it("rejects zero deposit", async () => {
      try {
        await program.methods
          .deposit(new BN(0))
          .accounts({ owner: payer.publicKey, mint, vaultState, ownerAta, vaultAta } as any)
          .rpc();
        expect.fail("should have thrown");
      } catch (e: any) {
        expect(e.message).to.include("ZeroAmount");
      }
    });

    it("rejects deposit exceeding max_deposit", async () => {
      try {
        await program.methods
          .deposit(new BN(MAX_DEPOSIT + 1))
          .accounts({ owner: payer.publicKey, mint, vaultState, ownerAta, vaultAta } as any)
          .rpc();
        expect.fail("should have thrown");
      } catch (e: any) {
        expect(e.message).to.include("DepositTooLarge");
      }
    });

    it("rejects signer that is not the owner", async () => {
      const attacker    = Keypair.generate();
      const attackerAta = await fundAta(conn, payer, mint, attacker.publicKey, 100_000);
      try {
        await program.methods
          .deposit(new BN(100_000))
          .accounts({
            owner:    attacker.publicKey,
            mint,
            vaultState, // still the legitimate vault
            ownerAta: attackerAta,
            vaultAta,
          } as any)
          .signers([attacker])
          .rpc();
        expect.fail("should have thrown");
      } catch (e: any) {
        // Seeds mismatch OR has_one violation
        expect(e.message).to.match(/Unauthorised|seeds constraint/i);
      }
    });
  });

  // ─── withdraw ────────────────────────────────────────────────────────────────
  describe("withdraw", () => {
    it("returns tokens to owner ATA", async () => {
      const before = await getTokenBalance(conn, ownerAta);

      await program.methods
        .withdraw(new BN(200_000))
        .accounts({ owner: payer.publicKey, mint, vaultState, vaultAta, ownerAta } as any)
        .rpc();

      const after = await getTokenBalance(conn, ownerAta);
      expect(Number(after - before)).to.equal(200_000);
    });

    it("rejects withdraw exceeding vault balance", async () => {
      try {
        await program.methods
          .withdraw(new BN(999_999_999))
          .accounts({ owner: payer.publicKey, mint, vaultState, vaultAta, ownerAta } as any)
          .rpc();
        expect.fail("should have thrown");
      } catch (e: any) {
        expect(e.message).to.include("InsufficientFunds");
      }
    });

    it("enforces daily withdrawal limit", async () => {
      // DAILY_LIMIT = 5_000_000
      // Current vault has 500k - 200k = 300k remaining

      // First withdrawal within limit (2M)
      await program.methods
        .withdraw(new BN(100_000))
        .accounts({ owner: payer.publicKey, mint, vaultState, vaultAta, ownerAta } as any)
        .rpc();

      // Second withdrawal still within limit (total 2.1M in window)
      await program.methods
        .withdraw(new BN(100_000))
        .accounts({ owner: payer.publicKey, mint, vaultState, vaultAta, ownerAta } as any)
        .rpc();

      // Try to exceed daily limit: already withdrawn 200k + 100k + 100k = 400k in total
      // Attempting 5M more would be 5.4M > DAILY_LIMIT (5M)
      // But vault only has 100k left, so we need a fresh deposit
      await program.methods
        .deposit(new BN(1_000_000))
        .accounts({ owner: payer.publicKey, mint, vaultState, ownerAta, vaultAta } as any)
        .rpc();

      // Now withdraw up to daily limit boundary
      await program.methods
        .withdraw(new BN(4_600_000)) // total: 400k + 4.6M = 5M (exactly at limit)
        .accounts({ owner: payer.publicKey, mint, vaultState, vaultAta, ownerAta } as any)
        .rpc();

      // Next withdrawal should fail
      try {
        await program.methods
          .withdraw(new BN(1))
          .accounts({ owner: payer.publicKey, mint, vaultState, vaultAta, ownerAta } as any)
          .rpc();
        expect.fail("should have thrown DailyLimitExceeded");
      } catch (e: any) {
        expect(e.message).to.include("DailyLimitExceeded");
      }
    });
  });

  // ─── delegate ────────────────────────────────────────────────────────────────
  describe("delegate", () => {
    let delegateKp:  Keypair;
    let delegateAta: anchor.web3.PublicKey;
    let delegateRec: anchor.web3.PublicKey;

    const ALLOWANCE   = 100_000;
    const EXPIRES_FUT = Math.floor(Date.now() / 1000) + 3600; // +1h
    const EXPIRES_PAS = Math.floor(Date.now() / 1000) - 1;    // already past

    before(async () => {
      delegateKp  = Keypair.generate();
      delegateAta = await fundAta(conn, payer, mint, delegateKp.publicKey, 0);
      [delegateRec] = deriveDelegatePDA(vaultState, delegateKp.publicKey);
    });

    it("adds a delegate record", async () => {
      await program.methods
        .addDelegate(new BN(ALLOWANCE), new BN(EXPIRES_FUT))
        .accounts({
          owner:          payer.publicKey,
          mint,
          vaultState,
          delegate:       delegateKp.publicKey,
          delegateRecord: delegateRec,
        } as any)
        .rpc();

      const rec = await program.account.delegateRecord.fetch(delegateRec);
      expect(rec.allowance.toNumber()).to.equal(ALLOWANCE);
      expect(rec.used.toNumber()).to.equal(0);
    });

    it("delegate can withdraw within allowance", async () => {
      const before = await getTokenBalance(conn, delegateAta);

      await program.methods
        .delegateWithdraw(new BN(50_000))
        .accounts({
          delegateSigner: delegateKp.publicKey,
          mint,
          owner:          payer.publicKey,
          vaultState,
          vaultAta,
          delegateAta,
          delegateRecord: delegateRec,
        } as any)
        .signers([delegateKp])
        .rpc();

      const after = await getTokenBalance(conn, delegateAta);
      expect(Number(after - before)).to.equal(50_000);
    });

    it("delegate cannot exceed allowance", async () => {
      try {
        await program.methods
          .delegateWithdraw(new BN(ALLOWANCE)) // would exceed 50k remaining
          .accounts({
            delegateSigner: delegateKp.publicKey,
            mint,
            owner:   payer.publicKey,
            vaultState,
            vaultAta,
            delegateAta,
            delegateRecord: delegateRec,
          } as any)
          .signers([delegateKp])
          .rpc();
        expect.fail("should have thrown");
      } catch (e: any) {
        expect(e.message).to.include("AllowanceExceeded");
      }
    });

    it("rejects expired delegate", async () => {
      // Create a second delegate with expired timestamp
      const exp2Kp  = Keypair.generate();
      await fundAta(conn, payer, mint, exp2Kp.publicKey, 0);
      const [exp2Rec] = deriveDelegatePDA(vaultState, exp2Kp.publicKey);

      // add_delegate with past expiry should fail
      try {
        await program.methods
          .addDelegate(new BN(10_000), new BN(EXPIRES_PAS))
          .accounts({
            owner: payer.publicKey, mint, vaultState,
            delegate: exp2Kp.publicKey, delegateRecord: exp2Rec,
          } as any)
          .rpc();
        expect.fail("should have thrown");
      } catch (e: any) {
        expect(e.message).to.include("DelegateExpired");
      }
    });

    it("owner can remove delegate and reclaim rent", async () => {
      await program.methods
        .removeDelegate()
        .accounts({
          owner: payer.publicKey, mint, vaultState,
          delegate: delegateKp.publicKey,
          delegateRecord: delegateRec,
        } as any)
        .rpc();

      try {
        await program.account.delegateRecord.fetch(delegateRec);
        expect.fail("account should be closed");
      } catch (_) {
        // account gone — expected
      }
    });
  });

  // ─── close ───────────────────────────────────────────────────────────────────
  describe("close", () => {
    it("rejects close when vault ATA has balance", async () => {
      try {
        await program.methods
          .closeVault()
          .accounts({ owner: payer.publicKey, mint, vaultState, vaultAta } as any)
          .rpc();
        expect.fail("should have thrown");
      } catch (e: any) {
        // ATA still has tokens from previous tests
        expect(e.message).to.match(/VaultNotEmpty|0x1/i);
      }
    });

    it("closes vault after full withdrawal", async () => {
      // drain vault first
      const balance = Number(await getTokenBalance(conn, vaultAta));
      if (balance > 0) {
        await program.methods
          .withdraw(new BN(balance))
          .accounts({ owner: payer.publicKey, mint, vaultState, vaultAta, ownerAta } as any)
          .rpc();
      }

      await program.methods
        .closeVault()
        .accounts({ owner: payer.publicKey, mint, vaultState, vaultAta } as any)
        .rpc();

      try {
        await program.account.vaultState.fetch(vaultState);
        expect.fail("account should be closed");
      } catch (_) {
        // expected
      }
    });
  });
});
