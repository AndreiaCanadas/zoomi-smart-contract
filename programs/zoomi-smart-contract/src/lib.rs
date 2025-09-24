use anchor_lang::prelude::*;

declare_id!("2j4NFfTwWjWukncAyLYLKo5GdrCY9f3xqiVNppzHuKMF");

#[program]
pub mod zoomi_smart_contract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
