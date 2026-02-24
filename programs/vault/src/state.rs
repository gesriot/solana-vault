use anchor_lang::prelude::*;

/// Central vault state account (PDA, seeds = [b"vault", owner])
#[account]
#[derive(Default)]
pub struct VaultState {
    /// Owner pubkey — the only signer allowed to deposit / withdraw freely
    pub owner: Pubkey,
    /// SPL mint this vault accepts
    pub mint: Pubkey,
    /// The vault's Associated Token Account (PDA-owned)
    pub vault_ata: Pubkey,
    /// Cumulative lifetime deposits (for analytics)
    pub total_deposited: u64,
    /// Cumulative lifetime withdrawals
    pub total_withdrawn: u64,
    /// Maximum single deposit (0 = unlimited)
    pub max_deposit: u64,
    /// Max tokens that can leave the vault in a 24-h window
    pub daily_withdraw_limit: u64,
    /// Amount already withdrawn in the current window
    pub withdrawn_today: u64,
    /// Unix timestamp of the start of the current 24-h window
    pub window_start: i64,
    /// Bump stored to avoid recomputing inside CPIs
    pub bump: u8,
    /// Whether the vault is locked (reentrancy guard)
    pub locked: bool,
}

impl VaultState {
    pub const LEN: usize = 8    // discriminator
        + 32 + 32 + 32          // owner, mint, vault_ata
        + 8 + 8                 // total_deposited, total_withdrawn
        + 8 + 8 + 8 + 8        // max_deposit, daily_withdraw_limit, withdrawn_today, window_start
        + 1 + 1; // bump, locked
}

/// Per-delegate record (PDA, seeds = [b"delegate", vault, delegate_pubkey])
#[account]
pub struct DelegateRecord {
    pub vault: Pubkey,
    pub delegate: Pubkey,
    pub allowance: u64,
    pub used: u64,
    pub expires_at: i64,
    pub bump: u8,
}

impl DelegateRecord {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 8 + 8 + 1;
}
