use anchor_lang::prelude::*;
use crate::state::{Rental, Rider, Scooter};
use crate::constants::PROTOCOL_FEE;

#[derive(Accounts)]
pub struct ExtendRentalPeriod<'info> {
    #[account(mut)]
    pub rider: Signer<'info>,
    #[account(mut)]
    pub rider_account: Account<'info, Rider>,
    #[account(mut)]
    pub scooter_account: Account<'info, Scooter>,
    #[account(
        mut,
        seeds = [rider_account.key().as_ref(), scooter_account.key().as_ref()],
        bump = rental_account.bump,
    )]
    pub rental_account: Account<'info, Rental>,
}
impl<'info> ExtendRentalPeriod<'info> {
    pub fn extend_rental_period(&mut self, additional_rental_period: u16) -> Result<()> {
        // Update rental period
        self.rental_account.rental_period += additional_rental_period;
        // Update total amount
        let mut new_amount = additional_rental_period * self.scooter_account.hourly_rate;
        new_amount += new_amount * PROTOCOL_FEE as u16 / 100;

        self.rental_account.total_amount += new_amount;
        Ok(())
    }
}