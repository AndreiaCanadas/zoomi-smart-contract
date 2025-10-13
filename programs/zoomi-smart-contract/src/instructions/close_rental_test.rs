use anchor_lang::prelude::*;
use crate::state::{Rental, RentalStatus, ScooterStatus, Scooter, Rider};
use anchor_spl::token::{Mint, TokenAccount, Token, close_account, CloseAccount, transfer_checked, TransferChecked};


#[derive(Accounts)]
pub struct CloseRentalTest<'info> {
    /// CHECK: Rider account    // TODO: Is this correct??
    #[account(mut)]
    pub rider: AccountInfo<'info>,
    #[account(
        mut,
        associated_token::mint = mint_usdc,
        associated_token::authority = rider,
    )]
    pub rider_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"rider", rider.key().as_ref()],
        bump = rider_account.bump,
    )]
    pub rider_account: Account<'info, Rider>,
    #[account(
        mut,
        constraint = scooter_account.status == ScooterStatus::Maintenance,
        seeds = [b"scooty", scooter_account.zoomi_device_pubkey.as_ref()],
        bump = scooter_account.bump,
    )]
    pub scooter_account: Account<'info, Scooter>,
    #[account(
        mut,
        close = rider,
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
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
impl<'info> CloseRentalTest<'info> {
    pub fn close_rental_test(&mut self) -> Result<()> {

        // Update scooter status to Available
        self.scooter_account.status = ScooterStatus::Available;

        // Signer seeds
        let signer_seeds: [&[&[u8]]; 1] = [&[
            self.rider_account.to_account_info().key.as_ref(),
            &self.scooter_account.to_account_info().key.as_ref(),
            &[self.rental_account.bump],
        ]];

        let cpi_program = self.token_program.to_account_info();

        // Empty vault
        let amount = self.vault.amount;
        let transfer_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_usdc.to_account_info(),
            to: self.rider_ata.to_account_info(),
            authority: self.rental_account.to_account_info(),
        };
        let transfer_cpi_ctx = CpiContext::new_with_signer(cpi_program.clone(), transfer_accounts, &signer_seeds);
        transfer_checked(transfer_cpi_ctx, amount, self.mint_usdc.decimals)?;


        // Close vault account
        let close_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.rider.to_account_info(),
            authority: self.rental_account.to_account_info(),
        };
        let close_cpi_ctx = CpiContext::new_with_signer(cpi_program, close_accounts, &signer_seeds);
        close_account(close_cpi_ctx)?;

        Ok(())
    }
}