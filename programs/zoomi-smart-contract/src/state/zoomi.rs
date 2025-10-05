use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Zoomi {
    pub admin: Pubkey,      // The wallet address of the administrator/authority
    pub treasury: Pubkey,   // The wallet address of the treasury
    pub fee: u8,            // The fee percentage for the treasury (% of total amount excluding collateral)
    pub collateral: u16,    // The collateral amount in USDC
    pub bump: u8,           // The bump of the admin account
}