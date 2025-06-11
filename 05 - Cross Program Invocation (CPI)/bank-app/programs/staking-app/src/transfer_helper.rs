use anchor_lang::{
    prelude::*,
    solana_program::{
        program::{invoke, invoke_signed},
        system_instruction::transfer,
    },
};

pub fn sol_transfer_from_user<'info>(
    signer: &Signer<'info>,
    destination: AccountInfo<'info>,
    system_program: &Program<'info, System>,
    amount: u64,
) -> Result<()> {
    let ix = transfer(signer.key, destination.key, amount);
    invoke(
        &ix,
        &[
            signer.to_account_info(),
            destination,
            system_program.to_account_info(),
        ],
    )?;
    Ok(())
}

pub fn sol_transfer_from_pda<'info>(
    source: AccountInfo<'info>,
    destination: AccountInfo<'info>,
    system_program: &Program<'info, System>,
    pda_seeds: &[&[&[u8]]],
    amount: u64,
) -> Result<()> {
    let ix = transfer(source.key, destination.key, amount);
    invoke_signed(
        &ix,
        &[source, destination, system_program.to_account_info()],
        pda_seeds,
    )?;
    Ok(())
}
