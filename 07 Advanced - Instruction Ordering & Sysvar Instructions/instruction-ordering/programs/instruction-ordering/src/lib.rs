use anchor_lang::prelude::*;
use solana_program::sysvar::instructions;

declare_id!("Your11111111111111111111111111111111111111");

#[program]
pub mod exercise {
    use super::*;

    // ---------------- Part 1: Instruction Ordering ----------------

    pub fn approve(ctx: Context<Approve>) -> Result<()> {
        // TODO: Implement approval logic (you can just log for now)
        msg!("Approval granted");
        Ok(())
    }

    pub fn execute(ctx: Context<Execute>, amount: u64) -> Result<()> {
        // TODO: Check that previous instruction was `approve`
        // - Use `instructions::load_current_index_checked` to get the current index
        // - Ensure there was at least one previous instruction
        // - Use `instructions::load_instruction_at_checked` to fetch the previous ix
        // - Verify:
        //     * previous_ix.program_id == crate::ID
        //     * first 8 bytes of previous_ix.data match the "approve" discriminator
        //       (hint: `hash(b"global:approve").to_bytes()[0..8]`)

        msg!("Executing with amount: {}", amount);
        Ok(())
    }

    // ---------------- Part 2: Large Data – Regular vs Zero-Copy ----------------

    pub fn initialize_large_approval_regular(
        ctx: Context<InitializeLargeApprovalRegular>,
    ) -> Result<()> {
        // TODO:
        // - Initialize a "regular" large account using `Account<LargeApprovalDataRegular>`
        // - Set the authority to `ctx.accounts.authority.key()`
        // - Zero out the approval_history array
        Ok(())
    }

    pub fn process_large_approval_regular(ctx: Context<ProcessLargeApprovalRegular>) -> Result<()> {
        // TODO:
        // - Get current timestamp from `Clock::get()?`
        // - Find the first empty slot (value == 0) in approval_history
        // - Write the timestamp there
        Ok(())
    }

    pub fn initialize_large_approval_zero_copy(
        ctx: Context<InitializeLargeApprovalZeroCopy>,
    ) -> Result<()> {
        // TODO:
        // - Use `ctx.accounts.approval_data.load_init()?` to get a zero-copy reference
        // - Set the authority (as bytes) and zero out the 512-element approval_history array
        Ok(())
    }

    pub fn process_large_approval_zero_copy(
        ctx: Context<ProcessLargeApprovalZeroCopy>,
    ) -> Result<()> {
        // TODO:
        // - Similar to the regular version, but using zero-copy:
        //   `let mut data = ctx.accounts.approval_data.load_mut()?;`
        // - Use `Clock::get()?` and write the timestamp into the first empty slot
        Ok(())
    }
}

// ---------------- Part 1 Accounts ----------------

#[derive(Accounts)]
pub struct Approve<'info> {
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Execute<'info> {
    pub authority: Signer<'info>,

    /// CHECK: Instructions sysvar
    // TODO: Add constraint to verify this is the instructions sysvar
    // Hint: `#[account(address = solana_program::sysvar::instructions::ID)]`
    pub instructions: UncheckedAccount<'info>,
}

// ---------------- Part 2: Regular Account<T> ----------------

// TODO: Adjust this length to be "large but still compiles" under BPF stack limits.
// Later, you can experiment with increasing it to see stack usage errors.
pub const REGULAR_HISTORY_LEN: usize = 128;

#[account]
pub struct LargeApprovalDataRegular {
    // TODO: Add fields:
    // - authority: Pubkey
    // - approval_history: [u64; REGULAR_HISTORY_LEN]
}

#[derive(Accounts)]
pub struct InitializeLargeApprovalRegular<'info> {
    #[account(
        init,
        payer = authority,
        // TODO: Set correct space: 8 + size_of::<LargeApprovalDataRegular>()
        space = 8,
        // TODO: Choose PDA seeds (e.g. b"approval_regular", authority key)
        seeds = [],
        bump
    )]
    pub approval_data: Account<'info, LargeApprovalDataRegular>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ProcessLargeApprovalRegular<'info> {
    #[account(
        mut,
        // TODO: Use the same seeds as in InitializeLargeApprovalRegular
        seeds = [],
        bump
    )]
    pub approval_data: Account<'info, LargeApprovalDataRegular>,

    pub authority: Signer<'info>,
}

// ---------------- Part 2: Zero-Copy AccountLoader<T> ----------------

// TODO:
// - Make this a zero-copy account: `#[account(zero_copy)]`
// - Add `#[repr(C)]` and derives needed for zero-copy (e.g. Copy, Clone, Default or bytemuck)
// - Add fields:
//     * authority: [u8; 32]
//     * approval_history: [u64; 512]   // full large array
pub struct LargeApprovalData {
    // TODO
}

#[derive(Accounts)]
pub struct InitializeLargeApprovalZeroCopy<'info> {
    #[account(
        init,
        payer = authority,
        // TODO: Set correct space: 8 + size_of::<LargeApprovalData>()
        space = 8,
        // TODO: Choose PDA seeds (e.g. b"approval_zero_copy", authority key)
        seeds = [],
        bump
    )]
    // TODO: Use AccountLoader<LargeApprovalData> instead of Account<...>
    pub approval_data: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ProcessLargeApprovalZeroCopy<'info> {
    #[account(
        mut,
        // TODO: Use the same seeds as in InitializeLargeApprovalZeroCopy
        seeds = [],
        bump
    )]
    // TODO: Use AccountLoader<LargeApprovalData>
    pub approval_data: UncheckedAccount<'info>,

    pub authority: Signer<'info>,
}

// ---------------- Errors ----------------

#[error_code]
pub enum ErrorCode {
    #[msg("Must approve before executing")]
    MustApproveFirst,
}
