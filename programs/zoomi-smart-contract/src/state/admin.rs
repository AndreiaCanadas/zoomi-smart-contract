use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Admin {
    pub admin: Pubkey,      // The wallet address of the administrator/authority
    pub fee: u8,           // The fee percentage for the treasury (% of total amount excluding collateral)
    pub treasury_bump: u8,  // The bump of the treasury account
    pub bump: u8,           // The bump of the admin account
}