use anchor_lang::{prelude::*, system_program};

declare_id!("EYdKY4wWuwNr7uVRQNBUEXeJyLCAatSELPck3quW7JvA");

pub mod transfer_helper;

#[program]
pub mod staking_app {
    use transfer_helper::{sol_transfer_from_pda, sol_transfer_from_user};

    use super::*;

    const STAKING_APR: u64 = 5; //5%
    const SECOND_PER_YEAR: u64 = 31_536_000;

    pub fn stake(ctx: Context<Stake>, amount: u64, is_stake: bool) -> Result<()> {
        let user_info = &mut ctx.accounts.user_info;

        let current_time: u64 = Clock::get()?.unix_timestamp.try_into().unwrap();
        let pass_time = if user_info.last_update_time == 0 {
            //just initialized
            0
        } else {
            current_time - user_info.last_update_time
        };

        user_info.amount += user_info.amount * STAKING_APR * pass_time / 100 / SECOND_PER_YEAR;
        user_info.last_update_time = current_time;

        if amount != 0 {
            if is_stake {
                sol_transfer_from_user(
                    &ctx.accounts.user,
                    ctx.accounts.staking_vault.to_account_info(),
                    &ctx.accounts.system_program,
                    amount,
                )?;

                user_info.amount += amount;
            } else {
                let pda_seeds: &[&[&[u8]]] = &[&[b"STAKING_VAULT", &[ctx.bumps.staking_vault]]];

                sol_transfer_from_pda(
                    ctx.accounts.staking_vault.to_account_info(),
                    ctx.accounts.user.to_account_info(),
                    &ctx.accounts.system_program,
                    pda_seeds,
                    amount,
                )?;

                user_info.amount -= amount;
            }
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Stake<'info> {
    /// CHECK:
    #[account(
        init_if_needed,
        payer = payer,
        seeds = [b"STAKING_VAULT"],
        bump,
        space = 0,
        owner = system_program::ID
    )]
    pub staking_vault: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        seeds = [b"USER_INFO", user.key().as_ref()],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<UserInfo>(),
    )]
    pub user_info: Box<Account<'info, UserInfo>>,

    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Default)]
pub struct UserInfo {
    pub amount: u64,
    pub last_update_time: u64,
}
