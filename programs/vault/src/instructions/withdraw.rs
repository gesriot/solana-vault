use crate::{errors::VaultError, events::WithdrawMade, state::VaultState};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

const DAY_SECONDS: i64 = 86_400;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"vault", owner.key().as_ref(), mint.key().as_ref()],
        bump  = vault_state.bump,
        has_one = owner @ VaultError::Unauthorised,
        has_one = mint  @ VaultError::Unauthorised,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        address = vault_state.vault_ata @ VaultError::Unauthorised,
    )]
    pub vault_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = owner_ata.owner == owner.key()  @ VaultError::Unauthorised,
        constraint = owner_ata.mint  == mint.key()   @ VaultError::Unauthorised,
    )]
    pub owner_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    require!(amount > 0, VaultError::ZeroAmount);

    let vault_state_ai = ctx.accounts.vault_state.to_account_info();
    let vault = &mut ctx.accounts.vault_state;
    require!(!vault.locked, VaultError::VaultLocked);

    let clock = Clock::get()?;

    // Roll the 24-h window if necessary
    if clock.unix_timestamp - vault.window_start >= DAY_SECONDS {
        vault.window_start    = clock.unix_timestamp;
        vault.withdrawn_today = 0;
    }

    if vault.daily_withdraw_limit > 0 {
        let new_today = vault.withdrawn_today
            .checked_add(amount)
            .ok_or(VaultError::Overflow)?;
        require!(new_today <= vault.daily_withdraw_limit, VaultError::DailyLimitExceeded);
        vault.withdrawn_today = new_today;
    }

    require!(ctx.accounts.vault_ata.amount >= amount, VaultError::InsufficientFunds);

    vault.locked = true;

    // PDA signer seeds
    let owner_key = vault.owner;
    let mint_key  = vault.mint;
    let bump      = vault.bump;
    let seeds     = &[b"vault", owner_key.as_ref(), mint_key.as_ref(), &[bump]];
    let signer    = &[&seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from:      ctx.accounts.vault_ata.to_account_info(),
            to:        ctx.accounts.owner_ata.to_account_info(),
            authority: vault_state_ai,
        },
        signer,
    );
    token::transfer(cpi_ctx, amount)?;

    vault.total_withdrawn = vault.total_withdrawn
        .checked_add(amount)
        .ok_or(VaultError::Overflow)?;
    vault.locked = false;

    emit!(WithdrawMade {
        vault: vault.key(),
        recipient: ctx.accounts.owner.key(),
        amount,
        by_delegate: false,
        timestamp: clock.unix_timestamp,
    });

    msg!("[vault] withdraw amount={} total_withdrawn={}", amount, vault.total_withdrawn);
    Ok(())
}
