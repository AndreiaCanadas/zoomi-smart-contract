use anchor_lang::prelude::*;
use crate::state::{Rental, Scooter, Rider, RentalStatus,ScooterStatus};
use crate::events::RentalEnded;
use crate::constants::SECONDS_IN_HOUR;

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
        seeds = [b"scooty", scooter_account.zoomi_device_pubkey.as_ref()],
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

        // Calculate actual usage vs paid rental period
        let paid_hours = self.rental_account.rental_period;
        let current_time = Clock::get()?.unix_timestamp;
        let actual_usage_seconds = current_time - self.rental_account.start_time;
        let actual_usage_hours = ((actual_usage_seconds + SECONDS_IN_HOUR as i64 - 1) / SECONDS_IN_HOUR as i64) as u16; // Round up
        
        // Calculate usage difference: positive = overused (penalty), negative = underused (refund)
        let usage_difference = actual_usage_hours as i16 - paid_hours as i16;
        self.rental_account.usage_adjustment = usage_difference;
        
        // If overused, add penalty to rider account
        if usage_difference > 0 {
            self.rider_account.penalties += usage_difference as u32;
        }

        // Update scooter status (Status to be updated to Available in close rental account, called by shopkeeper)
        self.scooter_account.status = ScooterStatus::Maintenance;

        // Update rider account (Points are to be updated in close rental account, called by shopkeeper)
        self.rider_account.is_renting = false;

        // Update rental account    (Shopkeeper to close rental account when everything is confirmed)
        self.rental_account.status = RentalStatus::Completed;

        emit!(RentalEnded {
            zoomi_device_pubkey: self.scooter_account.zoomi_device_pubkey,
        });

        Ok(())
    }
}