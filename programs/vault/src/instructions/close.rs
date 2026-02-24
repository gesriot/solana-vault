use crate::{errors::VaultError, events::VaultClosed, state::VaultState};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct CloseVault<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        close = owner,
        seeds = [b"vault", owner.key().as_ref(), mint.key().as_ref()],
        bump  = vault_state.bump,
        has_one = owner @ VaultError::Unauthorised,
        has_one = mint  @ VaultError::Unauthorised,
        constraint = vault_state.vault_ata == vault_ata.key() @ VaultError::Unauthorised,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        constraint = vault_ata.amount == 0 @ VaultError::VaultNotEmpty,
    )]
    pub vault_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CloseVault>) -> Result<()> {
    let vault    = &ctx.accounts.vault_state;
    let clock    = Clock::get()?;

    let owner_key = vault.owner;
    let mint_key  = vault.mint;
    let bump      = vault.bump;
    let seeds     = &[b"vault", owner_key.as_ref(), mint_key.as_ref(), &[bump]];
    let signer    = &[&seeds[..]];

    // Close the ATA and return rent to owner
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account:     ctx.accounts.vault_ata.to_account_info(),
            destination: ctx.accounts.owner.to_account_info(),
            authority:   ctx.accounts.vault_state.to_account_info(),
        },
        signer,
    );
    token::close_account(cpi_ctx)?;

    emit!(VaultClosed {
        vault:     ctx.accounts.vault_state.key(),
        owner:     owner_key,
        timestamp: clock.unix_timestamp,
    });

    msg!("[vault] vault closed owner={}", owner_key);
    Ok(())
}
