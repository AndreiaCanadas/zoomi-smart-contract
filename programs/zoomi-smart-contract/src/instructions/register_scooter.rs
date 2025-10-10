use anchor_lang::prelude::*;
use crate::state::{Scooter, ScooterStatus};

#[derive(Accounts)]
#[instruction(zoomi_device_pubkey: Pubkey)]
pub struct RegisterScooter<'info> {
    #[account(mut)]
    pub shopkeeper: Signer<'info>,
    #[account(
        init, 
        payer = shopkeeper, 
        space = 8 + Scooter::INIT_SPACE,
        seeds = [b"scooty", zoomi_device_pubkey.as_ref()],
        bump,
    )]
    pub scooter_account: Account<'info, Scooter>,
    pub system_program: Program<'info, System>,
}
impl<'info> RegisterScooter<'info> {
    pub fn register_scooter(&mut self, zoomi_device_pubkey: Pubkey, id: u32, shopkeeper_id: u32, hourly_rate: u16, bumps: &RegisterScooterBumps) -> Result<()> {
        self.scooter_account.set_inner(Scooter {
            id,
            shopkeeper_id,
            zoomi_device_pubkey,
            hourly_rate,
            status: ScooterStatus::Available,
            location_lat: 0,
            location_long: 0,
            location_timestamp: 0,
            bump: bumps.scooter_account,
        });
        Ok(())
    }
}