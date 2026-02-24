use crate::{errors::VaultError, events::DepositMade, state::VaultState};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"vault", owner.key().as_ref(), mint.key().as_ref()],
        bump  = vault_state.bump,
        has_one = owner @ VaultError::Unauthorised,
        has_one = mint   @ VaultError::Unauthorised,
    )]
    pub vault_state: Account<'info, VaultState>,

    /// Source — must be owned by the owner signer
    #[account(
        mut,
        constraint = owner_ata.owner == owner.key()   @ VaultError::Unauthorised,
        constraint = owner_ata.mint  == mint.key()    @ VaultError::Unauthorised,
    )]
    pub owner_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = vault_state.vault_ata @ VaultError::Unauthorised,
    )]
    pub vault_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    require!(amount > 0, VaultError::ZeroAmount);

    let vault = &mut ctx.accounts.vault_state;
    require!(!vault.locked, VaultError::VaultLocked);

    if vault.max_deposit > 0 {
        require!(amount <= vault.max_deposit, VaultError::DepositTooLarge);
    }

    // reentrancy lock
    vault.locked = true;

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from:      ctx.accounts.owner_ata.to_account_info(),
            to:        ctx.accounts.vault_ata.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        },
    );
    token::transfer(cpi_ctx, amount)?;

    vault.total_deposited = vault.total_deposited
        .checked_add(amount)
        .ok_or(VaultError::Overflow)?;
    vault.locked = false;

    let clock = Clock::get()?;
    emit!(DepositMade {
        vault: vault.key(),
        depositor: ctx.accounts.owner.key(),
        amount,
        total_deposited: vault.total_deposited,
        timestamp: clock.unix_timestamp,
    });

    msg!("[vault] deposit amount={} total_deposited={}", amount, vault.total_deposited);
    Ok(())
}
