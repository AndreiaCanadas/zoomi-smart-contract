use anchor_lang::prelude::*;
use crate::state::zoomi::Zoomi;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount}};


#[derive(Accounts)]
pub struct InitializeZoomi<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,   
    #[account(
        init, 
        payer = admin, 
        space = 8 + Zoomi::INIT_SPACE,
        seeds = [b"zoomi", admin.key().as_ref()],
        bump,
    )]
    pub zoomi_account: Account<'info, Zoomi>,

    #[account(mut)]
    pub mint_usdc: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = mint_usdc,
        associated_token::authority = zoomi_account,
    )]
    pub treasury: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}
impl<'info> InitializeZoomi<'info> {
    pub fn initialize_zoomi(&mut self, protocol_fee: u8, base_rate: u64, collateral: u64, bumps: &InitializeZoomiBumps) -> Result<()> {
        self.zoomi_account.set_inner(Zoomi { 
            admin: self.admin.key(),
            treasury: self.treasury.key(),      // TODO: Does it make sense to save the treasury account here ??
            protocol_fee,
            base_rate,
            collateral,
            bump: bumps.zoomi_account,
        });
        Ok(())
    }
}