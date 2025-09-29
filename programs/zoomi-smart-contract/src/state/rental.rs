use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Rental {
    pub rider: Pubkey,          // Rider wallet address
    pub scooter_id: u32,        // Scooter ID
    pub start_time: i64,        // Start time of rental
    pub end_time: i64,          // End time of rental
    pub total_amount: u16,      // Total amount in USDC (hourly rate * time + collateral amount)
    pub extra_time: u16,        // Extra time in hours
    pub status: RentalStatus,   // Rental status
    pub bump: u8,               // Bump of the rental account
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, InitSpace)]
pub enum RentalStatus {
    Active = 0,
    Completed = 1,
    Cancelled = 2,
}