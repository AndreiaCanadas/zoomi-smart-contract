use anchor_lang::prelude::*;
use crate::state::{Rental, Rider, Scooter};

#[derive(Accounts)]
pub struct ExtendRentalPeriod<'info> {
    #[account(mut)]
    pub rider: Signer<'info>,       // TBD: Will Rider be a signer ??
    #[account(
        seeds = [b"rider", rider.key().as_ref()],
        bump = rider_account.bump,
    )]
    pub rider_account: Account<'info, Rider>,
    #[account(
        seeds = [b"scooter", scooter_account.zoomi_device_pubkey.as_ref()],
        bump = scooter_account.bump,
    )]
    pub scooter_account: Account<'info, Scooter>,
    #[account(
        mut,
        seeds = [rider_account.key().as_ref(), scooter_account.key().as_ref()],
        bump = rental_account.bump,
    )]
    pub rental_account: Account<'info, Rental>,
}
impl<'info> ExtendRentalPeriod<'info> {
    pub fn extend_rental_period(&mut self, additional_rental_period: u16, additional_amount: u16) -> Result<()> {
        // Update rental period
        self.rental_account.rental_period += additional_rental_period;
        
        // Update total amount
        self.rental_account.total_amount += additional_amount;

        Ok(())
    }
}