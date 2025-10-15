use anchor_lang::prelude::*;
use crate::state::{Rental, Scooter, Rider, RentalStatus, ScooterStatus, Zoomi};
use crate::events::ScooterUnlocked;

use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount, transfer_checked, TransferChecked}};

#[derive(Accounts)]
pub struct StartRental<'info> {
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
        constraint = scooter_account.status == ScooterStatus::Available,
        seeds = [b"scooty", scooter_account.zoomi_device_pubkey.as_ref()],
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
    #[account(
        seeds = [b"zoomi", zoomi_account.admin.key().as_ref()],
        bump = zoomi_account.bump,
    )]
    pub zoomi_account: Account<'info, Zoomi>,

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
    pub fn start_rental(&mut self, rental_period: u16, bumps: &StartRentalBumps) -> Result<()> {

        let base_rate = self.zoomi_account.base_rate;                         // Base amount in USDC
        let hourly_rate = self.scooter_account.hourly_rate * rental_period as u64;   // Rental amount for the period in USDC
        let protocol_fee = self.zoomi_account.protocol_fee;                    // Protocol fee in %
        let collateral = self.zoomi_account.collateral;                       // Collateral in USDC

        let mut total_amount = (base_rate + hourly_rate) * (100 + protocol_fee as u64) / 100;
        total_amount += collateral;
      
        // Set rental account
        self.rental_account.set_inner(Rental {
            rider: self.rider.key(),
            scooter_id: self.scooter_account.id,
            start_time: Clock::get()?.unix_timestamp,
            rental_period,
            total_amount,
            status: RentalStatus::Active,
            bump: bumps.rental_account,
        });

        // Update scooter account
        self.scooter_account.status = ScooterStatus::Rented;

        // Update rider account
        self.rider_account.is_renting = true;

        // Transfer total amount from rider to vault
        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = TransferChecked {
            from: self.rider_ata.to_account_info(),
            mint: self.mint_usdc.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.rider.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(cpi_ctx, self.rental_account.total_amount as u64, self.mint_usdc.decimals)?;

        emit!(ScooterUnlocked {
            zoomi_device_pubkey: self.scooter_account.zoomi_device_pubkey,
            rental_duration: self.rental_account.rental_period,
        });

        Ok(())
    }
}