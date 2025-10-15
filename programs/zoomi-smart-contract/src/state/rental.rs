use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Rental {
    pub rider: Pubkey,          // Rider wallet address
    pub scooter_id: u32,        // Scooter ID
    pub start_time: i64,        // Start time of rental (unix timestamp)
    pub rental_period: u16,     // Rental period in hours
    pub total_amount: u64,      // Total amount in USDC (base rate + hourly rate * time + collateral + protocol fee)
    pub status: RentalStatus,   // Rental status
    pub bump: u8,               // Bump of the rental account
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, InitSpace)]
pub enum RentalStatus {
    Active = 0,
    Completed = 1,
    Cancelled = 2,
}