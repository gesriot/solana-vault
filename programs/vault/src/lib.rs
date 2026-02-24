#![allow(unexpected_cfgs)]

//! # Vault — Solana Anchor Program
//!
//! Secure SPL-token vault with:
//!  - owner deposit / withdraw
//!  - time-limited, capped delegate withdrawals
//!  - checked arithmetic, verified CPIs, canonical PDAs
//!  - on-chain events for off-chain observability

use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("31mDBe7jLM8UVoqfBCUpC5yKsijh3uoKekKgRR1Z7VCJ");

#[program]
pub mod vault {
    use super::*;

    /// Initialise a new vault with configurable limits.
    pub fn initialize(
        ctx: Context<Initialize>,
        max_deposit: u64,
        daily_withdraw_limit: u64,
    ) -> Result<()> {
        initialize::handler(ctx, max_deposit, daily_withdraw_limit)
    }

    /// Deposit SPL tokens from owner into the vault PDA ATA.
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        deposit::handler(ctx, amount)
    }

    /// Withdraw SPL tokens back to owner (full authority).
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        withdraw::handler(ctx, amount)
    }

    /// Grant a delegate capped, time-limited withdraw authority.
    pub fn add_delegate(ctx: Context<AddDelegate>, allowance: u64, expires_at: i64) -> Result<()> {
        delegate::add_handler(ctx, allowance, expires_at)
    }

    /// Revoke an existing delegate.
    pub fn remove_delegate(ctx: Context<RemoveDelegate>) -> Result<()> {
        delegate::remove_handler(ctx)
    }

    /// Delegate exercises partial withdrawal within allowance.
    pub fn delegate_withdraw(ctx: Context<DelegateWithdraw>, amount: u64) -> Result<()> {
        delegate::withdraw_handler(ctx, amount)
    }

    /// Close vault, burn rent to owner.
    pub fn close_vault(ctx: Context<CloseVault>) -> Result<()> {
        close::handler(ctx)
    }
}
