# solana-vault · v0.1.0

> **Web3 Track**  
> Solana Anchor program: secure SPL-token vault with delegate access.

---

## Problem

DeFi protocols need a custody primitive that:
1. Holds arbitrary SPL tokens for a single owner.
2. Grants time-limited, capped withdrawal rights to third parties (delegates).
3. Enforces daily withdrawal limits to bound damage from a compromised key.
4. Resists the entire OWASP Solana top-10 (missing signer checks, arbitrary CPI,
   re-initialisation, integer overflow, reentrancy, …).

---

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for a full diagram.

**Accounts**
- `VaultState` — PDA `[b"vault", owner, mint]` — central state & guard
- `DelegateRecord` — PDA `[b"delegate", vault_state, delegate]` — per-delegate allowance

**Instructions**
| Instruction | Who | What |
|---|---|---|
| `initialize` | owner | Create vault + ATA, set limits |
| `deposit` | owner | Transfer tokens owner→vault |
| `withdraw` | owner | Transfer tokens vault→owner (daily-limit enforced) |
| `add_delegate` | owner | Grant capped/timed delegate |
| `remove_delegate` | owner | Close delegate record, reclaim rent |
| `delegate_withdraw` | delegate | Withdraw within allowance & expiry (NOT subject to daily limit) |
| `close_vault` | owner | Close vault (must be empty) |

---

## Quick Start

### Prerequisites
```bash
# Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.18.22/install)"

# Anchor via AVM
cargo install --git https://github.com/coral-xyz/anchor avm --locked
avm install 0.30.1 && avm use 0.30.1

# Node deps
npm install
```

Or run the automated script:
```bash
chmod +x scripts/setup.sh && ./scripts/setup.sh
```

### Build
```bash
anchor build
```

### Test (localnet spun up automatically by Anchor)
```bash
anchor test
```

### Deploy to devnet
```bash
solana config set --url devnet
solana airdrop 2
anchor deploy --provider.cluster devnet
```

---

## How to Test

```bash
anchor test           # all tests (spins up test-validator automatically)
npx ts-mocha -p tsconfig.json tests/vault_property.ts    # property tests only
```

Fuzz targets (requires nightly):
```bash
cargo +nightly fuzz run fuzz_vault
```

---

## Metrics / Observability

| Metric | How to observe |
|---|---|
| Compute units per ix | `scripts/benchmark.sh` → grep "consumed X of" |
| Token flow | On-chain events `DepositMade`, `WithdrawMade` via `program.addEventListener` |
| Errors | Anchor error codes in `errors.rs`; all mapped to human messages |
| Delegate usage | `DelegateRecord.used / allowance` queryable any time |

All instructions emit structured `#[event]` logs parseable by any Solana indexer.

**Important:** The `daily_withdraw_limit` applies **only to owner withdrawals** via the `withdraw` instruction.
Delegate withdrawals via `delegate_withdraw` are bounded by their individual `allowance` and `expires_at` parameters,
but do NOT count against the vault's daily withdrawal limit. This design choice allows:
- Owner to set a global daily limit for their own withdrawals (risk mitigation for key compromise)
- Delegates to have independent, pre-authorized withdrawal caps (for service providers, sub-accounts, etc.)

If you need delegates to share the same daily limit pool, consider implementing a separate rate-limiter PDA.

---

## Security

See [SECURITY.md](SECURITY.md) for the full threat model (9 mitigated CVE classes).

## Testing Notes

See [TESTING.md](TESTING.md) for coverage scope and exact commands.

## Issue Tracker

Project backlog is tracked in [ISSUES.md](ISSUES.md).

---

## Reproducibility

```bash
# pin exact toolchain
rustup override set 1.79.0
# deterministic build
anchor build -- --locked
# deterministic JS deps
npm ci
```

Benchmark parameters are committed in `scripts/benchmark.sh`.  
All tests use fixed seeds + local test-validator — no mainnet state dependency.

---

## Changelog

### v0.1.0
- Initial vault: initialize, deposit, withdraw, delegate lifecycle, close
- 9-item threat model, integration + property tests, CI pipeline
