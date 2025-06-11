use anchor_lang::prelude::*;

pub mod constant;
pub mod error;
pub mod instructions;
pub mod state;
pub mod transfer_helper;

use instructions::*;

declare_id!("3q57ftWH75aKfxoNnV6Lu1n8LhV73xxKHAxPapL6Jvh7");

#[program]
pub mod bank_app {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        return Initialize::process(ctx);
    }

    pub fn deposit(ctx: Context<Deposit>, deposit_amount: u64) -> Result<()> {
        return Deposit::process(ctx, deposit_amount);
    }
}
