#![allow(deprecated, unexpected_cfgs)]

use anchor_lang::prelude::*;

mod state;
mod instructions;
mod constants;
mod errors;
mod events;

use instructions::*;
use state::ScooterStatus;

declare_id!("C3fDBNS8FdQTR6XYkMk2dYH1YgZgy5R7tj7jANs34gfA");

#[program]
pub mod zoomi_smart_contract {
    use super::*;

    pub fn initialize_zoomi(ctx: Context<InitializeZoomi>, protocol_fee: u8, base_rate: u64, collateral: u64) -> Result<()> {
        ctx.accounts.initialize_zoomi(protocol_fee, base_rate, collateral, &ctx.bumps)
    }

    pub fn register_rider(ctx: Context<RegisterRider>) -> Result<()> {
        ctx.accounts.register_rider(&ctx.bumps)
    }

    pub fn register_scooter(ctx: Context<RegisterScooter>, zoomi_device_pubkey: Pubkey, id: u32, shopkeeper_id: u32, hourly_rate: u64) -> Result<()> {
        ctx.accounts.register_scooter(zoomi_device_pubkey, id, shopkeeper_id, hourly_rate, &ctx.bumps)
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

    pub fn return_scooter(ctx: Context<ReturnScooter>) -> Result<()> {
        ctx.accounts.return_scooter()
    }

    pub fn close_rental(ctx: Context<CloseRental>, scooter_ok: bool) -> Result<()> {
        ctx.accounts.close_rental(scooter_ok)
    }

    // For testing purposes only
    pub fn close_rental_test(ctx: Context<CloseRentalTest>) -> Result<()> {
        ctx.accounts.close_rental_test()
    }

    // For testing purposes only
    pub fn close_zoomi(ctx: Context<CloseZoomi>) -> Result<()> {
        ctx.accounts.close_zoomi()
    }

}

