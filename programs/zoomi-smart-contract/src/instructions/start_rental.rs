use anchor_lang::prelude::*;
use crate::state::{Rental, Scooter, Rider, RentalStatus, ScooterStatus};

use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount}};

#[derive(Accounts)]
pub struct StartRental<'info> {
    #[account(mut)]
    pub rider: Signer<'info>,       // TBD: Will Rider be a signer ?? If not, how to make sure method is only called by backend or something when payment is confirmed ?
    #[account(
        seeds = [b"rider", rider.key().as_ref()],
        bump = rider_account.bump,
    )]
    pub rider_account: Account<'info, Rider>,
    #[account(
        mut,
        constraint = scooter_account.status == ScooterStatus::Available,
        seeds = [b"scooter", scooter_account.zoomi_device_pubkey.as_ref()],
        bump = scooter_account.bump,
    )]
    pub scooter_account: Account<'info, Scooter>,
    #[account(
        init, 
        payer = rider, 
        space = 8 + Rental::INIT_SPACE,
        seeds = [rider_account.key().as_ref(), scooter_account.key().as_ref()],
        bump,
    )]
    pub rental_account: Account<'info, Rental>,

    // TODO: Vault Account for USDC to be initialized in frontend ???? And total amount to be transferred in frontend ?? (Solana Pay ?)
    #[account(mut)]
    pub mint_usdc: Account<'info, Mint>,
    #[account(
        init,
        payer = rider,
        associated_token::mint = mint_usdc,
        associated_token::authority = rental_account,
    )]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}
impl<'info> StartRental<'info> {
    pub fn start_rental(&mut self, rental_period: u16, total_amount: u16, bumps: &StartRentalBumps) -> Result<()> {
      
        // Set rental account
        self.rental_account.set_inner(Rental {
            rider: self.rider.key(),
            scooter_id: self.scooter_account.id,
            start_time: Clock::get()?.unix_timestamp,
            rental_period,
            total_amount,
            penalty_time: 0,
            status: RentalStatus::Active,
            bump: bumps.rental_account,
        });

        // Update scooter account
        self.scooter_account.status = ScooterStatus::Rented;

        // Update rider account
        self.rider_account.is_renting = true;

        Ok(())
    }
}