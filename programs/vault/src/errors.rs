use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Deposit exceeds the configured maximum")]
    DepositTooLarge,
    #[msg("Withdrawal would exceed the daily limit")]
    DailyLimitExceeded,
    #[msg("Insufficient funds in vault")]
    InsufficientFunds,
    #[msg("Vault is locked — possible reentrancy attempt")]
    VaultLocked,
    #[msg("Delegate allowance exceeded")]
    AllowanceExceeded,
    #[msg("Delegate record has expired")]
    DelegateExpired,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Unauthorised signer")]
    Unauthorised,
    #[msg("Vault must be empty before closing")]
    VaultNotEmpty,
}
