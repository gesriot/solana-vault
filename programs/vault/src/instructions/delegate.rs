use crate::{
    errors::VaultError,
    events::{DelegateAdded, DelegateRemoved, WithdrawMade},
    state::{DelegateRecord, VaultState},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

// ─── AddDelegate ─────────────────────────────────────────────────────────────

#[derive(Accounts)]
#[instruction(allowance: u64, expires_at: i64)]
pub struct AddDelegate<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"vault", owner.key().as_ref(), mint.key().as_ref()],
        bump  = vault_state.bump,
        has_one = owner @ VaultError::Unauthorised,
    )]
    pub vault_state: Account<'info, VaultState>,

    /// CHECK: arbitrary pubkey we're granting access to
    pub delegate: UncheckedAccount<'info>,

    #[account(
        init,
        payer = owner,
        space = DelegateRecord::LEN,
        seeds = [b"delegate", vault_state.key().as_ref(), delegate.key().as_ref()],
        bump,
    )]
    pub delegate_record: Account<'info, DelegateRecord>,

    pub system_program: Program<'info, System>,
}

pub fn add_handler(ctx: Context<AddDelegate>, allowance: u64, expires_at: i64) -> Result<()> {
    require!(allowance > 0, VaultError::ZeroAmount);

    let clock = Clock::get()?;
    require!(
        expires_at > clock.unix_timestamp,
        VaultError::DelegateExpired
    );

    let rec = &mut ctx.accounts.delegate_record;
    rec.vault = ctx.accounts.vault_state.key();
    rec.delegate = ctx.accounts.delegate.key();
    rec.allowance = allowance;
    rec.used = 0;
    rec.expires_at = expires_at;
    rec.bump = ctx.bumps.delegate_record;

    emit!(DelegateAdded {
        vault: ctx.accounts.vault_state.key(),
        delegate: rec.delegate,
        allowance,
        expires_at,
    });

    msg!(
        "[vault] delegate added={} allowance={}",
        rec.delegate,
        allowance
    );
    Ok(())
}

// ─── RemoveDelegate ───────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct RemoveDelegate<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        seeds = [b"vault", owner.key().as_ref(), mint.key().as_ref()],
        bump  = vault_state.bump,
        has_one = owner @ VaultError::Unauthorised,
    )]
    pub vault_state: Account<'info, VaultState>,

    /// CHECK: the delegate being removed
    pub delegate: UncheckedAccount<'info>,

    #[account(
        mut,
        close = owner,
        seeds = [b"delegate", vault_state.key().as_ref(), delegate.key().as_ref()],
        bump  = delegate_record.bump,
        constraint = delegate_record.vault == vault_state.key() @ VaultError::Unauthorised,
    )]
    pub delegate_record: Account<'info, DelegateRecord>,

    pub system_program: Program<'info, System>,
}

pub fn remove_handler(ctx: Context<RemoveDelegate>) -> Result<()> {
    emit!(DelegateRemoved {
        vault: ctx.accounts.vault_state.key(),
        delegate: ctx.accounts.delegate.key(),
    });
    msg!("[vault] delegate removed={}", ctx.accounts.delegate.key());
    Ok(())
}

// ─── DelegateWithdraw ─────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct DelegateWithdraw<'info> {
    /// Must be the exact delegate pubkey stored in the record
    pub delegate_signer: Signer<'info>,

    pub mint: Account<'info, Mint>,

    /// CHECK: vault owner — used only in seed derivation, validated via has_one
    pub owner: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"vault", owner.key().as_ref(), mint.key().as_ref()],
        bump  = vault_state.bump,
        has_one = mint @ VaultError::Unauthorised,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        address = vault_state.vault_ata @ VaultError::Unauthorised,
    )]
    pub vault_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = delegate_ata.owner == delegate_signer.key() @ VaultError::Unauthorised,
        constraint = delegate_ata.mint  == mint.key()            @ VaultError::Unauthorised,
    )]
    pub delegate_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"delegate", vault_state.key().as_ref(), delegate_signer.key().as_ref()],
        bump  = delegate_record.bump,
        constraint = delegate_record.delegate == delegate_signer.key() @ VaultError::Unauthorised,
        constraint = delegate_record.vault    == vault_state.key()     @ VaultError::Unauthorised,
    )]
    pub delegate_record: Account<'info, DelegateRecord>,

    pub token_program: Program<'info, Token>,
}

pub fn withdraw_handler(ctx: Context<DelegateWithdraw>, amount: u64) -> Result<()> {
    require!(amount > 0, VaultError::ZeroAmount);

    let clock = Clock::get()?;
    let rec = &mut ctx.accounts.delegate_record;

    require!(
        clock.unix_timestamp < rec.expires_at,
        VaultError::DelegateExpired
    );

    let new_used = rec.used.checked_add(amount).ok_or(VaultError::Overflow)?;
    require!(new_used <= rec.allowance, VaultError::AllowanceExceeded);

    let vault_state_ai = ctx.accounts.vault_state.to_account_info();
    let vault = &mut ctx.accounts.vault_state;
    require!(!vault.locked, VaultError::VaultLocked);
    require!(
        ctx.accounts.vault_ata.amount >= amount,
        VaultError::InsufficientFunds
    );

    vault.locked = true;
    rec.used = new_used;

    let owner_key = vault.owner;
    let mint_key = vault.mint;
    let bump = vault.bump;
    let seeds = &[b"vault", owner_key.as_ref(), mint_key.as_ref(), &[bump]];
    let signer = &[&seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault_ata.to_account_info(),
            to: ctx.accounts.delegate_ata.to_account_info(),
            authority: vault_state_ai,
        },
        signer,
    );
    token::transfer(cpi_ctx, amount)?;

    vault.total_withdrawn = vault
        .total_withdrawn
        .checked_add(amount)
        .ok_or(VaultError::Overflow)?;
    vault.locked = false;

    emit!(WithdrawMade {
        vault: vault.key(),
        recipient: ctx.accounts.delegate_signer.key(),
        amount,
        by_delegate: true,
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "[vault] delegate_withdraw delegate={} amount={}",
        rec.delegate,
        amount
    );
    Ok(())
}
