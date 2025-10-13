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
    #[account(
        mut,
        associated_token::mint = mint_usdc,
        associated_token::authority = zoomi_account,
    )]
    pub treasury: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
impl<'info> CloseRental<'info> {
    pub fn close_rental(&mut self, inspection_score: u8) -> Result<()> {
        // Update scooter status to Available
        self.scooter_account.status = ScooterStatus::Available;

        // Handle usage adjustments: refunds for unused hours OR extra charges for overused hours
        // Apply same fee calculation as start_rental: hourly_rate * (100 + fee%) / 100
        let hourly_rate_with_fee = self.scooter_account.hourly_rate * (100 + self.zoomi_account.fee as u16) / 100;
        let mut refund_amount = 0u16;
        let mut extra_charge = 0u16;
        
        if self.rental_account.usage_adjustment < 0 {
            // Refund for unused hours (including protocol fee)
            let unused_hours = (-self.rental_account.usage_adjustment) as u16;
            refund_amount = unused_hours * hourly_rate_with_fee;
        } else if self.rental_account.usage_adjustment > 0 {
    
            // Extra charge for overused hours (including protocol fee, deducted from collateral)
            let overused_hours = self.rental_account.usage_adjustment as u16;
            extra_charge = overused_hours * hourly_rate_with_fee;
        }

        // Calculate base collateral distribution based on inspection score (0-100)
        // 80-100: Full collateral refund to rider
        // 50-79: Partial collateral (proportional) to rider, rest to shopkeeper  
        // 0-49: No collateral refund, all goes to shopkeeper
        let base_collateral_to_rider = if inspection_score >= 80 {
            self.zoomi_account.collateral
        } else if inspection_score >= 50 {
            // Proportional: 50 = 0%, 79 = 96.67% (roughly linear)
            (self.zoomi_account.collateral * (inspection_score - 50) as u16) / 30
        } else {
            0
        };
        
        // Deduct extra charges from rider's collateral share
        let collateral_to_rider = if base_collateral_to_rider >= extra_charge {
            base_collateral_to_rider - extra_charge
        } else {
            0 // Not enough collateral to cover extra charges
        };
        let collateral_to_shopkeeper = self.zoomi_account.collateral - collateral_to_rider;

        // Calculate actual rental amount: original amount - refunds + extra charges - collateral
        let actual_usage_amount = self.rental_account.total_amount - self.zoomi_account.collateral - refund_amount + extra_charge;
        
        // Update rider account points (based on actual rental without collateral)
        self.rider_account.points += actual_usage_amount as u32;
        
        // Signer seeds
        let signer_seeds: [&[&[u8]]; 1] = [&[
            self.rider_account.to_account_info().key.as_ref(),
            &self.scooter_account.to_account_info().key.as_ref(),
            &[self.rental_account.bump],
        ]];

        let cpi_program = self.token_program.to_account_info();

        // Transfer fee from vault to treasury (based on actual usage including extra charges)
        let fee = actual_usage_amount * self.zoomi_account.fee as u16 / 100;
        let transfer_fee_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_usdc.to_account_info(),
            to: self.treasury.to_account_info(),
            authority: self.rental_account.to_account_info(),
        };
        let transfer_fee_cpi_ctx = CpiContext::new_with_signer(cpi_program.clone(), transfer_fee_accounts, &signer_seeds);
        transfer_checked(transfer_fee_cpi_ctx, fee as u64, self.mint_usdc.decimals)?;

        // Transfer rental amount (after fee) from vault to shopkeeper + collateral penalty
        // Note: extra_charge is already included in actual_usage_amount and fee calculation
        let rent_to_shopkeeper = actual_usage_amount - fee + collateral_to_shopkeeper;
        let transfer_rent_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_usdc.to_account_info(),
            to: self.shopkeeper_ata.to_account_info(),
            authority: self.rental_account.to_account_info(),
        };
        let transfer_rent_cpi_ctx = CpiContext::new_with_signer(cpi_program.clone(), transfer_rent_accounts, &signer_seeds);
        transfer_checked(transfer_rent_cpi_ctx, rent_to_shopkeeper as u64, self.mint_usdc.decimals)?;

        // Refund unused hours + allowed collateral to rider
        let total_refund_to_rider = refund_amount + collateral_to_rider;
        if total_refund_to_rider > 0 {
            let transfer_refund_accounts = TransferChecked {
                from: self.vault.to_account_info(),
                mint: self.mint_usdc.to_account_info(),
                to: self.rider_ata.to_account_info(),
                authority: self.rental_account.to_account_info(),
            };
            let transfer_refund_cpi_ctx = CpiContext::new_with_signer(cpi_program.clone(), transfer_refund_accounts, &signer_seeds);
            transfer_checked(transfer_refund_cpi_ctx, total_refund_to_rider as u64, self.mint_usdc.decimals)?;
        }

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