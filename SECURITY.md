# Security — Threat Model & Known Pitfalls

## Scope

This document covers the `vault` Anchor program. It assumes:
- The **Solana runtime** and **SPL Token program** are trusted.
- The **RPC node** may be adversarial (use a local/trusted node for production).

---

## Threat Model

| Asset | Value | Threat |
|---|---|---|
| Vault ATA tokens | High | Arbitrary withdrawal by attacker |
| VaultState PDA | Medium | State manipulation, re-initialisation |
| DelegateRecord PDA | Low-Medium | Allowance bypass, expiry skip |
| Rent lamports | Low | Griefing via fake account creation |

---

## Mitigated Vulnerabilities

### 1. Missing Signer Check
**Risk:** Anyone calls `withdraw` without being the owner.  
**Mitigation:** `has_one = owner @ VaultError::Unauthorised` on `VaultState`.
Every instruction that mutates funds requires the matching `Signer<'info>`.

### 2. Arbitrary CPI (fake token program)
**Risk:** Attacker passes a malicious account as `token_program`.  
**Mitigation:** All CPI accounts use `Program<'info, Token>` — Anchor verifies
the account key equals `spl_token::ID` at deserialisation time.

### 3. Missing Owner Check (account substitution)
**Risk:** Attacker passes their own `VaultState` whose owner is themselves,
then calls `deposit` targeting the victim's vault ATA.  
**Mitigation:** PDA seeds include `owner.key()` — seed derivation is deterministic;
wrong owner → wrong PDA → `seeds` constraint fails.

### 4. Reinitialization Attack
**Risk:** Overwriting an existing vault's parameters.  
**Mitigation:** `init` constraint (not `init_if_needed`) — fails if the account
already has a non-zero discriminator.

### 5. Arithmetic Overflow / Underflow
**Risk:** Wrapping arithmetic on `u64` balances causes phantom balance.  
**Mitigation:** All accumulations use `.checked_add()` / `.checked_sub()` with
`ok_or(VaultError::Overflow)`. `overflow-checks = true` in release profile as defence-in-depth.

### 6. Reentrancy
**Risk:** CPI into vault during token transfer lets attacker re-enter.  
**Mitigation:** `vault.locked = true` set **before** every CPI, cleared after.
Any re-entrant call hits `require!(!vault.locked, VaultError::VaultLocked)`.

### 7. Stale / Expired Delegate
**Risk:** Delegate continues to withdraw after agreed expiry.  
**Mitigation:** `require!(clock.unix_timestamp < rec.expires_at, VaultError::DelegateExpired)`
checked on every `delegate_withdraw`.

### 8. PDA Bump Canonicality
**Risk:** Non-canonical bump allows collision with attacker-controlled account.  
**Mitigation:** `bump` is stored in `VaultState.bump` at `init` time (Anchor
always uses the canonical / highest bump). Subsequent calls reference
`vault_state.bump` directly, guaranteeing canonical derivation.

### 9. Type Confusion / Account Substitution via Discriminator
**Risk:** Attacker passes a different Anchor account type in the same slot.
**Mitigation:** Anchor automatically checks the 8-byte discriminator on every
`Account<'info, T>` deserialisation.

### 10. Stale Account Data After CPI
**Risk:** Reading cached account data after CPI may give stale values if the CPI modified the account.
**Mitigation:** In this program, we perform balance checks **before** CPI (preflight validation).
After CPI, we only update our own `VaultState` counters and don't re-read external account data.
The Token Program enforces the actual balance constraints during the CPI itself.
**Note:** If post-CPI validation were needed, use `.reload()?` to fetch fresh data from the runtime.

---

## Known Limitations / Out-of-Scope

- **Oracle manipulation** — no price feeds used.
- **Front-running** — Solana's single-leader model reduces but does not eliminate ordering risk.
- **Multi-sig owner** — not implemented in v0.x; use a Squads multisig as owner.
- **Token-2022 extensions** — not tested with transfer-fee or confidential-transfer mints.
- **Delegate daily rate limiting** — `daily_withdraw_limit` applies only to owner withdrawals.
  Delegates are bounded by their individual `allowance`, which does NOT count against the daily limit.
  This is a design choice: delegates have pre-authorized caps set at grant time. If you need
  delegates to share a global rate limit, implement a separate rate-limiter PDA.

---

## Responsible Disclosure

Open a **private GitHub Security Advisory** or email `security@example.com`.
