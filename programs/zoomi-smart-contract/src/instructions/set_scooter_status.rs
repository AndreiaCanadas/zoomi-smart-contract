use anchor_lang::prelude::*;
use crate::state::{Scooter, ScooterStatus};

#[derive(Accounts)]
pub struct SetScooterStatus<'info> {
    #[account(mut)]
    pub shopkeeper: Signer<'info>,
    #[account(
        mut,
        seeds = [b"scooter", scooter_account.id.to_le_bytes().as_ref(), scooter_account.shopkeeper_id.to_le_bytes().as_ref()],
        bump = scooter_account.bump,
    )]
    pub scooter_account: Account<'info, Scooter>,
}
impl<'info> SetScooterStatus<'info> {
    pub fn set_scooter_status(&mut self, status: ScooterStatus) -> Result<()> {
        self.scooter_account.status = status;
        Ok(())
    }
}
