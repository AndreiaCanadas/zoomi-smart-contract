use anchor_lang::prelude::*;
use crate::state::{Rental, Scooter, Rider, RentalStatus};
use crate::constants::COLLATERAL_AMOUNT;

#[derive(Accounts)]
pub struct InitializeRental<'info> {
    #[account(mut)]
    pub rider: Signer<'info>,
    #[account(mut)]
    pub rider_account: Account<'info, Rider>,
    #[account(mut)]
    pub scooter_account: Account<'info, Scooter>,
    #[account(
        init, 
        payer = rider, 
        space = 8 + Rental::INIT_SPACE,
        seeds = [rider_account.key().as_ref(), scooter_account.key().as_ref()],
        bump,
    )]
    pub rental_account: Account<'info, Rental>,
    pub system_program: Program<'info, System>,
}
impl<'info> InitializeRental<'info> {
    pub fn initialize_rental(&mut self, end_time: i64, rental_period: u16, bumps: &InitializeRentalBumps) -> Result<()> {
        self.rental_account.set_inner(Rental {
            rider: self.rider_account.key(),
            scooter_id: self.scooter_account.id,
            start_time: Clock::get()?.unix_timestamp,
            end_time,
            total_amount: rental_period * self.scooter_account.hourly_rate + COLLATERAL_AMOUNT,
            extra_time: 0,
            status: RentalStatus::Active,
            bump: bumps.rental_account,
        });
        Ok(())
    }
}