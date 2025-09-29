use anchor_lang::prelude::*;
use crate::state::Scooter;

#[derive(Accounts)]
pub struct UpdateScooterLocation<'info> {
    #[account(mut)]
    pub scooter_device: Signer<'info>,  // TODO: How to make sure is the correct signer?
    #[account(
        mut,
        seeds = [b"scooter", scooter_account.id.to_le_bytes().as_ref(), scooter_account.shopkeeper_id.to_le_bytes().as_ref()],
        bump = scooter_account.bump,
    )]
    pub scooter_account: Account<'info, Scooter>,
}
impl<'info> UpdateScooterLocation<'info> {
    pub fn update_scooter_location(&mut self, location_lat: i32, location_long: i32) -> Result<()> {
        self.scooter_account.location_lat = location_lat;
        self.scooter_account.location_long = location_long;
        self.scooter_account.location_timestamp = Clock::get()?.unix_timestamp;
        Ok(())
    }
}