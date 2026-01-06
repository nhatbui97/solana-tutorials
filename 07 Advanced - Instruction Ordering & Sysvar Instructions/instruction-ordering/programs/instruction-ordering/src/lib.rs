use anchor_lang::prelude::*;
use solana_program::sysvar::instructions;

declare_id!("9tM3F4Fg8SxR3Z1YR9J4yvQqHNYKvJrwXj5WXbNxPq8m");

#[program]
pub mod instruction_ordering {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        // Get current instruction index
        let current_index =
            instructions::load_current_index_checked(&ctx.accounts.instructions.to_account_info())?;

        // Must be first (index 0)
        require!(current_index == 0, ErrorCode::MustBeFirstInstruction);

        // Initialize state
        let state = &mut ctx.accounts.state;
        state.authority = ctx.accounts.authority.key();
        state.counter = 0;
        state.is_finalized = false;

        msg!("Initialized as first instruction");
        Ok(())
    }

    pub fn increment(ctx: Context<Increment>) -> Result<()> {
        ctx.accounts.state.counter += 1;
        msg!("Counter incremented to: {}", ctx.accounts.state.counter);
        Ok(())
    }

    pub fn finalize(ctx: Context<Finalize>) -> Result<()> {
        // TODO: Get current instruction index using load_current_index_checked
        // Hint: Pass ctx.accounts.instructions.to_account_info() as the parameter
        let current_index = todo!();

        // TODO: Count total instructions in transaction
        // Loop through all instructions until load_instruction_at_checked returns Err
        let mut total_instructions = 0;
        loop {
            // TODO: Try to load instruction at index total_instructions
            // If Ok(_), increment total_instructions
            // If Err(_), break the loop
            todo!()
        }

        // TODO: Check if current instruction is the last one
        // The last instruction has index == total_instructions - 1
        // Use require! macro with ErrorCode::MustBeLastInstruction
        todo!();

        // Mark as finalized
        ctx.accounts.state.is_finalized = true;
        msg!("Finalized as last instruction");

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8 + 1,
        seeds = [b"state"],
        bump
    )]
    pub state: Account<'info, State>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,

    /// CHECK: Instructions sysvar
    #[account(address = solana_program::sysvar::instructions::ID)]
    pub instructions: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct Increment<'info> {
    #[account(
        mut,
        seeds = [b"state"],
        bump
    )]
    pub state: Account<'info, State>,
}

#[derive(Accounts)]
pub struct Finalize<'info> {
    #[account(
        mut,
        seeds = [b"state"],
        bump
    )]
    pub state: Account<'info, State>,

    /// CHECK: Instructions sysvar
    #[account(address = solana_program::sysvar::instructions::ID)]
    pub instructions: UncheckedAccount<'info>,
}

#[account]
pub struct State {
    pub authority: Pubkey,
    pub counter: u64,
    pub is_finalized: bool,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Initialize must be the first instruction")]
    MustBeFirstInstruction,
    #[msg("Finalize must be the last instruction")]
    MustBeLastInstruction,
}
