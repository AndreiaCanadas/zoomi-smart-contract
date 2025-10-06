use anchor_lang::prelude::*;
use crate::state::{Rental, Scooter, Rider, RentalStatus, Zoomi,ScooterStatus};

use anchor_spl::token::{Mint, Token, TokenAccount, transfer_checked, TransferChecked};

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
    #[account(
        mut,
        seeds = [b"zoomi", zoomi_account.admin.key().as_ref()],
        bump = zoomi_account.bump,
    )]
    pub zoomi_account: Account<'info, Zoomi>,

    #[account(mut)]
    pub mint_usdc: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_usdc,
        associated_token::authority = rental_account,
    )]
    pub vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_usdc,
        associated_token::authority = zoomi_account,
    )]
    pub treasury: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
impl<'info> EndRental<'info> {
    pub fn end_rental(&mut self) -> Result<()> {

        // TODOs:
        // Check if rental period has passed (add penalties and subtract collateral)
        // Have a status and methods for scooter to be checked for damage (if damage, subtract collateral) ??
        // Close accounts here or only after shopkeeper confirms the scooter is in good condition ??
        // Transfer rental amount from vault to shopkeeper

        
        // Transfer fee from vault to treasury
        // let mut fee = self.rental_account.total_amount - self.zoomi_account.collateral;
        // fee = fee * self.zoomi_account.fee as u16 / 100;

        // let cpi_program = self.system_program.to_account_info();
        // let cpi_accounts = TransferChecked {
        //     from: self.vault.to_account_info(),
        //     mint: self.mint_usdc.to_account_info(),
        //     to: self.treasury.to_account_info(),
        //     authority: self.rental_account.to_account_info(),
        // };
        // let signer_seeds: [&[&[u8]]; 1] = [&[
        //     self.rider_account.to_account_info().key.as_ref(),
        //     &self.scooter_account.to_account_info().key.as_ref(),
        //     &[self.rental_account.bump],
        // ]];
        // let cpi_ctx = CpiContext::new_with_signer(
        //     cpi_program,
        //     cpi_accounts,
        //     &signer_seeds,
        // );
        // transfer_checked(cpi_ctx, fee as u64, self.mint_usdc.decimals)?;


        // Update scooter status (Status to be updated to Available in close rental account, called by shopkeeper)
        self.scooter_account.status = ScooterStatus::Maintenance;

        // Update rider account (Points are to be updated in close rental account, called by shopkeeper)
        self.rider_account.is_renting = false;

        // Update rental account    (Shopkeeper to close rental account when everything is confirmed)
        self.rental_account.status = RentalStatus::Completed;

        Ok(())
    }
}