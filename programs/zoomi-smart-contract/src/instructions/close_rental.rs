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
    pub fn close_rental(&mut self, scooter_ok: bool) -> Result<()> {
        
        // Get remaining amount in vault (collateral remaining after return_scooter)
        let vault_balance = self.vault.amount;
        
        // Tranfer vault balance based on scooter status

        // Signer seeds for vault transfers
        let signer_seeds: [&[&[u8]]; 1] = [&[
            self.rider_account.to_account_info().key.as_ref(),
            self.scooter_account.to_account_info().key.as_ref(),
            &[self.rental_account.bump],
        ]];

        let cpi_program = self.token_program.to_account_info();

        // OK (true): All collateral to rider
        if scooter_ok {
            let transfer_rider_accounts = TransferChecked {
                from: self.vault.to_account_info(),
                mint: self.mint_usdc.to_account_info(),
                to: self.rider_ata.to_account_info(),
                authority: self.rental_account.to_account_info(),
            };
            let transfer_rider_ctx = CpiContext::new_with_signer(
                cpi_program.clone(),
                transfer_rider_accounts,
                &signer_seeds
            );
            transfer_checked(transfer_rider_ctx, vault_balance, self.mint_usdc.decimals)?;
        }else{
            // NOK (false): All collateral to shopkeeper
            let transfer_shopkeeper_accounts = TransferChecked {
                from: self.vault.to_account_info(),
                mint: self.mint_usdc.to_account_info(),
                to: self.shopkeeper_ata.to_account_info(),
                authority: self.rental_account.to_account_info(),
            };
            let transfer_shopkeeper_ctx = CpiContext::new_with_signer(
                cpi_program.clone(),
                transfer_shopkeeper_accounts,
                &signer_seeds
            );
            transfer_checked(transfer_shopkeeper_ctx, vault_balance, self.mint_usdc.decimals)?;
        }

        // Close vault account
        let close_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.rider.to_account_info(),
            authority: self.rental_account.to_account_info(),
        };
        let close_ctx = CpiContext::new_with_signer(cpi_program, close_accounts, &signer_seeds);
        close_account(close_ctx)?;

        Ok(())
    }
}