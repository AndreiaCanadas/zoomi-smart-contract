use anchor_lang::prelude::*;

use crate::state::{Rental, RentalStatus, Rider, Scooter, ScooterStatus, Zoomi};

use anchor_spl::token::{Mint, TokenAccount, Token, transfer_checked, TransferChecked, close_account, CloseAccount};

#[derive(Accounts)]
pub struct CloseRental<'info> {
    #[account(mut)]
    pub shopkeeper: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = mint_usdc,
        associated_token::authority = shopkeeper,
    )]
    pub shopkeeper_ata: Account<'info, TokenAccount>,
    #[account(
        seeds = [b"zoomi", zoomi_account.admin.key().as_ref()],
        bump = zoomi_account.bump,
    )]
    pub zoomi_account: Account<'info, Zoomi>,
    #[account(
        mut,
        constraint = scooter_account.status == ScooterStatus::Maintenance,
        seeds = [b"scooty", scooter_account.zoomi_device_pubkey.as_ref()],
        bump = scooter_account.bump,
    )]
    pub scooter_account: Account<'info, Scooter>,
    #[account(
        mut,
        seeds = [b"rider", rider.key().as_ref()],
        bump = rider_account.bump,
    )]
    pub rider_account: Account<'info, Rider>,
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
    #[account(
        mut,
        associated_token::mint = mint_usdc,
        associated_token::authority = zoomi_account,
    )]
    pub treasury: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

// TODO: Add handling NOK case
// TODOs:
// Check if rental period has passed (add penalties and subtract collateral)
// Have a status and methods for scooter to be checked for damage (if damage, subtract collateral) ??
// Close accounts here or only after shopkeeper confirms the scooter is in good condition ??

impl<'info> CloseRental<'info> {
    pub fn close_rental(&mut self) -> Result<()> {
        // Update scooter status to Available
        self.scooter_account.status = ScooterStatus::Available;

        // Update rider account
        self.rider_account.points += (self.rental_account.total_amount - self.zoomi_account.collateral) as u32;
        
        // Signer seeds
        let signer_seeds: [&[&[u8]]; 1] = [&[
            self.rider_account.to_account_info().key.as_ref(),
            &self.scooter_account.to_account_info().key.as_ref(),
            &[self.rental_account.bump],
        ]];

        // Transfer fee from vault to treasury
        let mut fee = self.rental_account.total_amount - self.zoomi_account.collateral;
        fee = fee * self.zoomi_account.fee as u16 / 100;

        let cpi_program = self.system_program.to_account_info();
        let transfer_fee_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_usdc.to_account_info(),
            to: self.treasury.to_account_info(),
            authority: self.rental_account.to_account_info(),
        };
        let transfer_fee_cpi_ctx = CpiContext::new_with_signer(cpi_program.clone(), transfer_fee_accounts, &signer_seeds);
        transfer_checked(transfer_fee_cpi_ctx, fee as u64, self.mint_usdc.decimals)?;

        // Transfer rental amount from vault to shopkeeper
        let rent = self.rental_account.total_amount - fee - self.zoomi_account.collateral;

        let transfer_rent_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_usdc.to_account_info(),
            to: self.shopkeeper_ata.to_account_info(),
            authority: self.rental_account.to_account_info(),
        };
        let transfer_rent_cpi_ctx = CpiContext::new_with_signer(cpi_program.clone(), transfer_rent_accounts, &signer_seeds);
        transfer_checked(transfer_rent_cpi_ctx, rent as u64, self.mint_usdc.decimals)?;

        // Refund collateral from vault to rider
        let transfer_collateral_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_usdc.to_account_info(),
            to: self.rider_ata.to_account_info(),
            authority: self.rental_account.to_account_info(),
        };
        let transfer_collateral_cpi_ctx = CpiContext::new_with_signer(cpi_program.clone(), transfer_collateral_accounts, &signer_seeds);
        transfer_checked(transfer_collateral_cpi_ctx, self.zoomi_account.collateral as u64, self.mint_usdc.decimals)?;

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