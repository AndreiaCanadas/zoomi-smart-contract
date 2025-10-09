use anchor_lang::prelude::*;
use crate::state::Zoomi;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount}};

#[derive(Accounts)]
pub struct CloseZoomi<'info> {
    
    #[account(mut)]
    pub admin: Signer<'info>,   
    #[account(
        mut, 
        close = admin, 
        seeds = [b"zoomi", admin.key().as_ref()],
        bump,
    )]
    pub zoomi_account: Account<'info, Zoomi>,

    #[account(mut)]
    pub mint_usdc: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_usdc,
        associated_token::authority = zoomi_account,
    )]
    pub treasury: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}
impl<'info> CloseZoomi<'info> {
    pub fn close_zoomi(&mut self) -> Result<()> {
        
        Ok(())
    }
}