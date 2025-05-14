use anchor_lang::prelude::*;

declare_id!("GDGNBNAhHGmMKcxVxXBTTJ8xytmdjNuFWsr2igqhck27");

#[program]
pub mod my_first_anchor_project {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
