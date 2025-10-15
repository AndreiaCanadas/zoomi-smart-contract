use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Zoomi {
    pub admin: Pubkey,      // The wallet address of the administrator/authority
    pub treasury: Pubkey,   // The wallet address of the treasury
    pub protocol_fee: u8,   // The fee percentage for the treasury (% of rental fees)
    pub base_rate: u64,     // The base rate in USDC
    pub collateral: u64,    // The collateral amount in USDC
    pub bump: u8,           // The bump of the admin account
}