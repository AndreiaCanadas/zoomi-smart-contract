use anchor_lang::prelude::*;
use crate::state::{Rental, Rider, Scooter, Zoomi};
use anchor_spl::token::{Mint, Token, TokenAccount, transfer_checked, TransferChecked};

#[derive(Accounts)]
pub struct ExtendRentalPeriod<'info> {
    #[account(mut)]
    pub rider: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = mint_usdc,
        associated_token::authority = rider,
    )]
    pub rider_ata: Account<'info, TokenAccount>,
    #[account(
        seeds = [b"rider", rider.key().as_ref()],
        bump = rider_account.bump,
    )]
    pub rider_account: Account<'info, Rider>,
    #[account(
        seeds = [b"scooty", scooter_account.zoomi_device_pubkey.as_ref()],
        bump = scooter_account.bump,
    )]
    pub scooter_account: Account<'info, Scooter>,
    #[account(
        mut,
        seeds = [rider_account.key().as_ref(), scooter_account.key().as_ref()],
        bump = rental_account.bump,
    )]
    pub rental_account: Account<'info, Rental>,
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
    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}
impl<'info> ExtendRentalPeriod<'info> {
    pub fn extend_rental_period(&mut self, additional_rental_period: u16) -> Result<()> {

        let additional_amount = (additional_rental_period * self.scooter_account.hourly_rate) * (1 + self.zoomi_account.fee as u16 / 100);

        // Update rental period
        self.rental_account.rental_period += additional_rental_period;
        
        // Update total amount
        self.rental_account.total_amount += additional_amount;


        // Transfer total amount from rider to vault
        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = TransferChecked {
            from: self.rider_ata.to_account_info(),
            mint: self.mint_usdc.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.rider.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(cpi_ctx, additional_amount as u64, self.mint_usdc.decimals)?;


        Ok(())
    }
}