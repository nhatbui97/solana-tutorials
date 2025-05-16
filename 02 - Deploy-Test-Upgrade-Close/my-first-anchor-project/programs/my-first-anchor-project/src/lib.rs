use anchor_lang::prelude::*;

declare_id!("GDGNBNAhHGmMKcxVxXBTTJ8xytmdjNuFWsr2igqhck27");

#[program]
pub mod my_first_anchor_project {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let name = "Nhat";
        let age = 23;

        msg!("My name is {}", name);
        msg!("I'm {} years old", age);
        msg!("This is my first anchor project!");

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
