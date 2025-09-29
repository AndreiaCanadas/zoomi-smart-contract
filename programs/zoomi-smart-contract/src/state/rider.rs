use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Rider {
    pub wallet: Pubkey,        // Rider wallet address
    pub id_status: bool,       // ID status (true = verified, false = not verified)
    pub points: u32,           // Rider points
    pub penalties: u32,        // Rider penalties
    pub is_renting: bool,      // Renting status
    pub bump: u8,               // Bump of the rider account
}