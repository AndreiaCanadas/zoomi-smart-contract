use anchor_lang::prelude::*;
use crate::state::Zoomi;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount, close_account, CloseAccount}};

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

        let cpi_program = self.token_program.to_account_info();
        // Signer seeds for vault transfers
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"zoomi",
            self.admin.to_account_info().key.as_ref(),
            &[self.zoomi_account.bump],
        ]];

        // close treasury account
        let close_accounts = CloseAccount {
            account: self.treasury.to_account_info(),
            destination: self.admin.to_account_info(),
            authority: self.zoomi_account.to_account_info(),
        };
        let close_ctx = CpiContext::new_with_signer(cpi_program, close_accounts, &signer_seeds);
        close_account(close_ctx)?;
        
        Ok(())
    }
}