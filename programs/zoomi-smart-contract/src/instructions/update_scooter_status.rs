use anchor_lang::prelude::*;
use crate::state::{Scooter, ScooterStatus};

#[derive(Accounts)]
pub struct UpdateScooterStatus<'info> {
    #[account(mut)]
    pub scooter_device: Signer<'info>,
    #[account(
        mut,
        constraint = scooter_account.zoomi_device_pubkey == scooter_device.key(),
        seeds = [b"scooter", scooter_account.zoomi_device_pubkey.as_ref()],
        bump = scooter_account.bump,
    )]
    pub scooter_account: Account<'info, Scooter>,
}
impl<'info> UpdateScooterStatus<'info> {
    pub fn update_scooter_status(&mut self, status: ScooterStatus) -> Result<()> {
        self.scooter_account.status = status;
        Ok(())
    }
}