use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Scooter {
    pub id: u32,                        // Scooter unique ID from DB
    pub shopkeeper_id: u32,             // Shopkeeper unique ID from DB
    pub zoomi_device_pubkey: Pubkey,    // Zoomi device public key associated with the scooter
    pub hourly_rate: u64,               // Price per hour in USDC
    pub status: ScooterStatus,          // Scooter status
    pub location_lat: i32,              // i32 = f32 * 1000000
    pub location_long: i32,             // i32 = f32 * 1000000
    pub location_timestamp: i64,        // Timestamp of last location update
    pub bump: u8,                       // Bump of the scooter account
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, InitSpace)]
pub enum ScooterStatus {
    Available = 0,
    Booked = 1,
    Rented = 2,
    Maintenance = 3,
}