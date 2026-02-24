# Architecture

## Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Client (TypeScript)                       │
│   program.methods.deposit() / withdraw() / addDelegate() …      │
└────────────────────────────┬────────────────────────────────────┘
                             │ Anchor IDL / RPC
┌────────────────────────────▼────────────────────────────────────┐
│              vault program  (Fg6PaFp…)                           │
│                                                                  │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────────────┐ │
│  │  initialize  │   │   deposit    │   │       withdraw       │ │
│  └──────────────┘   └──────┬───────┘   └──────────┬───────────┘ │
│                             │                       │            │
│  ┌──────────────────────────▼───────────────────────▼──────────┐ │
│  │                    VaultState PDA                            │ │
│  │  seeds=["vault", owner, mint]  bump stored on-chain         │ │
│  │  owner · mint · vault_ata · total_deposited · locked …      │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │                 DelegateRecord PDA (per delegate)             │ │
│  │  seeds=["delegate", vault_state, delegate_pubkey]           │ │
│  │  allowance · used · expires_at                               │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  CPI → spl-token program (Transfer / CloseAccount)               │
└─────────────────────────────────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                       Vault ATA                                  │
│       Associated Token Account owned by VaultState PDA          │
└─────────────────────────────────────────────────────────────────┘
```

## Account Layout

### VaultState (size: 154 bytes)
| Field | Type | Description |
|---|---|---|
| owner | Pubkey | Sole authority for deposit/withdraw |
| mint | Pubkey | SPL mint accepted by this vault |
| vault_ata | Pubkey | Canonical ATA of this PDA |
| total_deposited | u64 | Lifetime deposits (analytics) |
| total_withdrawn | u64 | Lifetime withdrawals (analytics) |
| max_deposit | u64 | Max single deposit (0 = unlimited) |
| daily_withdraw_limit | u64 | 24-h rolling withdraw cap |
| withdrawn_today | u64 | Already withdrawn this window |
| window_start | i64 | Unix timestamp of window open |
| bump | u8 | Canonical PDA bump |
| locked | bool | Reentrancy guard |

### DelegateRecord (size: 97 bytes)
| Field | Type | Description |
|---|---|---|
| vault | Pubkey | Parent vault |
| delegate | Pubkey | Authorised pubkey |
| allowance | u64 | Total tokens delegate may withdraw |
| used | u64 | Amount already withdrawn |
| expires_at | i64 | Unix expiry |
| bump | u8 | Canonical bump |

## Instruction Flow

```
initialize → deposit ─┐
                       ├─ [withdraw | delegate_withdraw] ─ close_vault
add_delegate ──────────┘
remove_delegate
```

## CPI Safety

All CPIs go to `spl_token::ID` (enforced by `Program<'info, Token>`).
Vault PDA signs via `CpiContext::new_with_signer` using seeds
`["vault", owner_key, mint_key, &[bump]]` — no external account can
forge this signature.
