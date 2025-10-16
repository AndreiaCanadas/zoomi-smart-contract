use anchor_lang::prelude::*;
use crate::state::{Rental, Scooter, Rider, RentalStatus, ScooterStatus, Zoomi};
use crate::events::RentalEnded;
use crate::constants::SECONDS_IN_HOUR;
use anchor_spl::token::{Mint, Token, TokenAccount, transfer_checked, TransferChecked};

#[derive(Accounts)]
pub struct ReturnScooter<'info> {
    #[account(mut)]
    pub rider: Signer<'info>,
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
    #[account(
        mut,
        associated_token::mint = mint_usdc,
        associated_token::authority = shopkeeper,
    )]
    pub shopkeeper_ata: Account<'info, TokenAccount>,
    /// CHECK: Shopkeeper account    // TODO: Is this correct??
    #[account(mut)]
    pub shopkeeper: AccountInfo<'info>,
    #[account(
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
}
impl<'info> ReturnScooter<'info> {
    pub fn return_scooter(&mut self) -> Result<()> {

        // Calculate actual usage vs paid rental period
        let paid_hours = self.rental_account.rental_period;
        let current_time = Clock::get()?.unix_timestamp;
        let actual_usage_seconds = current_time - self.rental_account.start_time;
        let actual_usage_hours = ((actual_usage_seconds + SECONDS_IN_HOUR as i64 - 1) / SECONDS_IN_HOUR as i64) as u16; // Round up

        // Calculate final rental amount
        let mut final_rental_amount = self.rental_account.rental_amount;

        if actual_usage_hours < paid_hours {
            // Refund unused hours (refund to rider)
            final_rental_amount -= ((paid_hours - actual_usage_hours) as u64) * self.scooter_account.hourly_rate;
        } else if actual_usage_hours > paid_hours {
            // Charge extra hours (take from collateral)
            final_rental_amount += ((actual_usage_hours - paid_hours) as u64) * self.scooter_account.hourly_rate;
            // Add penalty points for overuse
            self.rider_account.penalties += (actual_usage_hours - paid_hours) as u32;
        }
        // Else, no adjustment needed to final rental amount
        let final_protocol_fee_amount = final_rental_amount * self.zoomi_account.protocol_fee as u64 / 100u64;

        // Signer seeds for vault transfers
        let signer_seeds: [&[&[u8]]; 1] = [&[
            self.rider_account.to_account_info().key.as_ref(),
            self.scooter_account.to_account_info().key.as_ref(),
            &[self.rental_account.bump],
        ]];

        // Transfer protocol fee to treasury
        let transfer_fee_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_usdc.to_account_info(),
            to: self.treasury.to_account_info(),
            authority: self.rental_account.to_account_info(),
        };
        let transfer_fee_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_fee_accounts,
            &signer_seeds
        );
        transfer_checked(transfer_fee_ctx, final_protocol_fee_amount as u64, self.mint_usdc.decimals)?;

        // Transfer rental amount to shopkeeper
        let transfer_rental_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_usdc.to_account_info(),
            to: self.shopkeeper_ata.to_account_info(),
            authority: self.rental_account.to_account_info(),
        };
        let transfer_rental_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_rental_accounts,
            &signer_seeds
        );
        transfer_checked(transfer_rental_ctx, final_rental_amount as u64, self.mint_usdc.decimals)?;

        // Refund unused hours to rider if applicable
        if actual_usage_hours < paid_hours {
            let unused_rental = ((paid_hours - actual_usage_hours) as u64) * self.scooter_account.hourly_rate;
            let unused_fee = unused_rental * self.zoomi_account.protocol_fee as u64 / 100;
            let amount_to_refund = unused_rental + unused_fee;
            
            let transfer_refund_accounts = TransferChecked {
                from: self.vault.to_account_info(),
                mint: self.mint_usdc.to_account_info(),
                to: self.rider_ata.to_account_info(),
                authority: self.rental_account.to_account_info(),
            };
            let transfer_refund_ctx = CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                transfer_refund_accounts,
                &signer_seeds
            );
            transfer_checked(transfer_refund_ctx, amount_to_refund, self.mint_usdc.decimals)?;
        }

        // Update account states
        self.scooter_account.status = ScooterStatus::Maintenance;
        self.rider_account.is_renting = false;
        self.rider_account.points += final_rental_amount as u32;
        self.rental_account.status = RentalStatus::Completed;

        emit!(RentalEnded {
            zoomi_device_pubkey: self.scooter_account.zoomi_device_pubkey,
        });

        Ok(())
    }
}