use anchor_lang::prelude::*;
use crate::state::{Scooter, ScooterStatus};

#[derive(Accounts)]
#[instruction(id: u32)]
pub struct InitializeScooter<'info> {
    #[account(mut)]
    pub shopkeeper: Signer<'info>,
    #[account(
        init, 
        payer = shopkeeper, 
        space = 8 + Scooter::INIT_SPACE,
        seeds = [b"scooter", shopkeeper.key().as_ref(), id.to_le_bytes().as_ref()],
        bump,
    )]
    pub scooter_account: Account<'info, Scooter>,
    pub system_program: Program<'info, System>,
}
impl<'info> InitializeScooter<'info> {
    pub fn initialize_scooter(&mut self, id: u32, shopkeeper_id: u32, hourly_rate: u16, bumps: &InitializeScooterBumps) -> Result<()> {
        self.scooter_account.set_inner(Scooter {
            id,
            shopkeeper_id,
            hourly_rate,
            status: ScooterStatus::Available,   // TODO: Initial status can be changed in future
            location_lat: 0,
            location_long: 0,
            location_timestamp: 0,
            bump: bumps.scooter_account,
        });
        Ok(())
    }
}