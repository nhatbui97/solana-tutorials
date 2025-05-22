use anchor_lang::prelude::*;

use crate::{
    constant::{BANK_INFO_SEED, USER_RESERVE_SEED},
    error::BankAppError,
    state::{BankInfo, UserReserve},
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        seeds = [BANK_INFO_SEED],
        bump
    )]
    pub bank_info: Box<Account<'info, BankInfo>>,

    #[account(
        mut,
        seeds = [USER_RESERVE_SEED, user.key().as_ref()],
        bump,
    )]
    pub user_reserve: Box<Account<'info, UserReserve>>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn process(ctx: Context<Withdraw>, withdraw_amount: u64) -> Result<()> {
        if ctx.accounts.bank_info.is_paused {
            return Err(BankAppError::BankAppPaused.into());
        }

        let pda_seeds: &[&[&[u8]]] = &[&[BANK_INFO_SEED, &[ctx.accounts.bank_info.bump]]];
        // Your code here

        Ok(())
    }
}
