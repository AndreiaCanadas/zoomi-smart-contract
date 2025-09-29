use anchor_lang::prelude::*;

mod state;
mod instructions;
mod constants;
mod errors;

use instructions::*;
use state::ScooterStatus;

declare_id!("2j4NFfTwWjWukncAyLYLKo5GdrCY9f3xqiVNppzHuKMF");

#[program]
pub mod zoomi_smart_contract {
    use super::*;

    pub fn register_rider(ctx: Context<RegisterRider>) -> Result<()> {
        ctx.accounts.register_rider(&ctx.bumps)
    }

    pub fn register_scooter(ctx: Context<RegisterScooter>, id: u32, shopkeeper_id: u32, hourly_rate: u16) -> Result<()> {
        ctx.accounts.register_scooter(id, shopkeeper_id, hourly_rate, &ctx.bumps)
    }

    pub fn start_rental(ctx: Context<StartRental>, rental_period: u16) -> Result<()> {
        ctx.accounts.start_rental(rental_period, &ctx.bumps)
    }

    pub fn extend_rental_period(ctx: Context<ExtendRentalPeriod>, additional_rental_period: u16) -> Result<()> {
        ctx.accounts.extend_rental_period(additional_rental_period)
    }

    pub fn update_scooter_location(ctx: Context<UpdateScooterLocation>, location_lat: i32, location_long: i32) -> Result<()> {
        ctx.accounts.update_scooter_location(location_lat, location_long)
    }

    pub fn set_scooter_status(ctx: Context<SetScooterStatus>, status: ScooterStatus) -> Result<()> {
        ctx.accounts.set_scooter_status(status)
    }

    pub fn update_scooter_status(ctx: Context<UpdateScooterStatus>, status: ScooterStatus) -> Result<()> {
        ctx.accounts.update_scooter_status(status)
    }

}

