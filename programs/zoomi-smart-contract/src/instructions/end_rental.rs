use anchor_lang::prelude::*;
use crate::state::{Rental, Scooter, Rider, RentalStatus,ScooterStatus};

#[derive(Accounts)]
pub struct EndRental<'info> {
    #[account(mut)]
    pub rider: Signer<'info>,
    #[account(
        mut,
        seeds = [b"rider", rider.key().as_ref()],
        bump = rider_account.bump,
    )]
    pub rider_account: Account<'info, Rider>,
    #[account(
        mut,
        constraint = scooter_account.status == ScooterStatus::Rented,
        seeds = [b"scooter", scooter_account.zoomi_device_pubkey.as_ref()],
        bump = scooter_account.bump,
    )]
    pub scooter_account: Account<'info, Scooter>,
    #[account(
        mut,
        constraint = rental_account.status == RentalStatus::Active,
        seeds = [rider_account.key().as_ref(), scooter_account.key().as_ref()],
        bump,
    )]
    pub rental_account: Account<'info, Rental>,
}
impl<'info> EndRental<'info> {
    pub fn end_rental(&mut self) -> Result<()> {

        // Update scooter status (Status to be updated to Available in close rental account, called by shopkeeper)
        self.scooter_account.status = ScooterStatus::Maintenance;

        // Update rider account (Points are to be updated in close rental account, called by shopkeeper)
        self.rider_account.is_renting = false;

        // Update rental account    (Shopkeeper to close rental account when everything is confirmed)
        self.rental_account.status = RentalStatus::Completed;

        Ok(())
    }
}