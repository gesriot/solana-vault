use crate::{events::VaultInitialised, state::VaultState};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    pub mint: Account<'info, Mint>,

    /// PDA vault state (seeds validated by Anchor constraint)
    #[account(
        init,
        payer = owner,
        space = VaultState::LEN,
        seeds = [b"vault", owner.key().as_ref(), mint.key().as_ref()],
        bump
    )]
    pub vault_state: Account<'info, VaultState>,

    /// Vault's ATA, owned by vault_state PDA
    #[account(
        init,
        payer = owner,
        associated_token::mint = mint,
        associated_token::authority = vault_state,
    )]
    pub vault_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<Initialize>,
    max_deposit: u64,
    daily_withdraw_limit: u64,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault_state;
    let clock = Clock::get()?;

    vault.owner = ctx.accounts.owner.key();
    vault.mint = ctx.accounts.mint.key();
    vault.vault_ata = ctx.accounts.vault_ata.key();
    vault.max_deposit = max_deposit;
    vault.daily_withdraw_limit = daily_withdraw_limit;
    vault.window_start = clock.unix_timestamp;
    vault.bump = ctx.bumps.vault_state;
    vault.locked = false;

    emit!(VaultInitialised {
        owner: vault.owner,
        mint: vault.mint,
        max_deposit,
        daily_withdraw_limit,
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "[vault] initialised owner={} mint={}",
        vault.owner,
        vault.mint
    );
    Ok(())
}
