use anchor_lang::prelude::*;

#[event]
pub struct VaultInitialised {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub max_deposit: u64,
    pub daily_withdraw_limit: u64,
    pub timestamp: i64,
}

#[event]
pub struct DepositMade {
    pub vault: Pubkey,
    pub depositor: Pubkey,
    pub amount: u64,
    pub total_deposited: u64,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawMade {
    pub vault: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub by_delegate: bool,
    pub timestamp: i64,
}

#[event]
pub struct DelegateAdded {
    pub vault: Pubkey,
    pub delegate: Pubkey,
    pub allowance: u64,
    pub expires_at: i64,
}

#[event]
pub struct DelegateRemoved {
    pub vault: Pubkey,
    pub delegate: Pubkey,
}

#[event]
pub struct VaultClosed {
    pub vault: Pubkey,
    pub owner: Pubkey,
    pub timestamp: i64,
}
