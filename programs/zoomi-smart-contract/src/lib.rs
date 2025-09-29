use anchor_lang::prelude::*;

mod state;
mod instructions;
mod constants;
mod errors;

use instructions::*;

declare_id!("2j4NFfTwWjWukncAyLYLKo5GdrCY9f3xqiVNppzHuKMF");

#[program]
pub mod zoomi_smart_contract {
    use super::*;

    pub fn initialize_rider(ctx: Context<InitializeRider>) -> Result<()> {
        ctx.accounts.initialize_rider(&ctx.bumps)
    }

}

