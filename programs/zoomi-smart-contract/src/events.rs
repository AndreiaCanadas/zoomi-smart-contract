use anchor_lang::prelude::*;

#[event]
pub struct ScooterUnlocked {
    pub zoomi_device_pubkey: Pubkey,
    pub rental_duration: u16,
}

#[event]
pub struct RentalExtended {
    pub zoomi_device_pubkey: Pubkey,
    pub additional_rental_period: u16,
}