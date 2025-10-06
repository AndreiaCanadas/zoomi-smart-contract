use anchor_lang::prelude::*;

use crate::state::{Rental, RentalStatus, Rider, Scooter, ScooterStatus, Zoomi};

#[derive(Accounts)]
pub struct CloseRental<'info> {
    #[account(mut)]
    pub shopkeeper: Signer<'info>,
    #[account(
        seeds = [b"zoomi", zoomi_account.admin.key().as_ref()],
        bump = zoomi_account.bump,
    )]
    pub zoomi_account: Account<'info, Zoomi>,
    #[account(
        mut,
        constraint = scooter_account.status == ScooterStatus::Maintenance,
        seeds = [b"scooter", scooter_account.zoomi_device_pubkey.as_ref()],
        bump = scooter_account.bump,
    )]
    pub scooter_account: Account<'info, Scooter>,
    #[account(
        mut,
        seeds = [b"rider", rider_account.key().as_ref()],
        bump = rider_account.bump,
    )]
    pub rider_account: Account<'info, Rider>,
    #[account(
        mut,
        close = shopkeeper,
        constraint = rental_account.status == RentalStatus::Completed,
        seeds = [rider_account.key().as_ref(), scooter_account.key().as_ref()],
        bump,
    )]
    pub rental_account: Account<'info, Rental>,

    #[account(mut)]
    pub mint_usdc: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_usdc,
        associated_token::authority = rental_account,
    )]
    pub vault: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
}

// TODO: Add handling NOK case
impl<'info> CloseRental<'info> {
    pub fn close_rental(&mut self) -> Result<()> {
        // Update scooter status to Available
        self.scooter_account.status = ScooterStatus::Available;

        // Update rider account
        self.rider_account.points += (self.rental_account.total_amount - self.zoomi_account.collateral) as u32;
        Ok(())
    }
}