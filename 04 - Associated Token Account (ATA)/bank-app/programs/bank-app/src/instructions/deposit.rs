use anchor_lang::prelude::*;

use crate::{
    constant::{BANK_INFO_SEED, USER_RESERVE_SEED},
    error::BankAppError,
    state::{BankInfo, UserReserve},
    transfer_helper::sol_transfer_from_user,
};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        seeds = [BANK_INFO_SEED],
        bump
    )]
    pub bank_info: Box<Account<'info, BankInfo>>,

    #[account(
        init_if_needed,
        seeds = [USER_RESERVE_SEED, user.key().as_ref()],
        bump,
        payer = user,
        space = 8 + std::mem::size_of::<UserReserve>(),
    )]
    pub user_reserve: Box<Account<'info, UserReserve>>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn process(ctx: Context<Deposit>, deposit_amount: u64) -> Result<()> {
        if ctx.accounts.bank_info.is_paused {
            return Err(BankAppError::BankAppPaused.into());
        }

        let user_reserve = &mut ctx.accounts.user_reserve;

        sol_transfer_from_user(
            &ctx.accounts.user,
            ctx.accounts.bank_info.to_account_info(),
            &ctx.accounts.system_program,
            deposit_amount,
        )?;

        user_reserve.deposited_amount += deposit_amount;

        Ok(())
    }
}
