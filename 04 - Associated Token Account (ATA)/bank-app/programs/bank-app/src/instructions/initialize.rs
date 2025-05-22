use anchor_lang::prelude::*;

use crate::{constant::BANK_INFO_SEED, state::BankInfo};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [BANK_INFO_SEED],
        bump,
        payer = authority,
        space = 8 + std::mem::size_of::<BankInfo>(),
    )]
    pub bank_info: Box<Account<'info, BankInfo>>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn process(ctx: Context<Initialize>) -> Result<()> {
        let bank_info = &mut ctx.accounts.bank_info;

        bank_info.authority = ctx.accounts.authority.key();
        bank_info.is_paused = false;
        bank_info.bump = ctx.bumps.bank_info;

        msg!("bank app initialized!");
        Ok(())
    }
}
