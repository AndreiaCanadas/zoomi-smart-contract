use anchor_lang::prelude::*;
use crate::state::{Rental, Scooter, Rider, RentalStatus, Admin};
use crate::constants::COLLATERAL_AMOUNT;

#[derive(Accounts)]
pub struct StartRental<'info> {
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
    #[account(
        seeds = [b"zoomi", admin_account.admin.key().as_ref()],
        bump = admin_account.bump,
    )]
    pub admin_account: Account<'info, Admin>,
    #[account(
        seeds = [b"treasury", admin_account.key().as_ref()],
        bump = admin_account.treasury_bump,
    )]
    pub treasury: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}
impl<'info> StartRental<'info> {
    pub fn start_rental(&mut self, rental_period: u16, bumps: &StartRentalBumps) -> Result<()> {
        let mut total_amount = rental_period * self.scooter_account.hourly_rate;
        total_amount += total_amount * self.admin_account.fee as u16 / 100;
        total_amount += COLLATERAL_AMOUNT;
        
        self.rental_account.set_inner(Rental {
            rider: self.rider_account.key(),
            scooter_id: self.scooter_account.id,
            start_time: Clock::get()?.unix_timestamp,
            rental_period,
            total_amount,
            extra_time: 0,
            status: RentalStatus::Active,
            bump: bumps.rental_account,
        });
        Ok(())
    }
}