use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct BankInfo {
    pub authority: Pubkey,
    pub is_paused: bool,
    pub bump: u8,
}

#[account]
#[derive(Default)]
pub struct UserReserve {
    pub deposited_amount: u64,
}
