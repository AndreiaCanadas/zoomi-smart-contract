use anchor_lang::prelude::*;
use crate::state::Rider;

#[derive(Accounts)]
pub struct InitializeRider<'info> {
    #[account(mut)]
    pub rider: Signer<'info>,
    #[account(
        init, 
        payer = rider, 
        space = 8 + Rider::INIT_SPACE,
        seeds = [b"rider", rider.key().as_ref()],
        bump,
    )]
    pub rider_account: Account<'info, Rider>,
    pub system_program: Program<'info, System>,
}
impl<'info> InitializeRider<'info> {
    pub fn initialize_rider(&mut self, bumps: &InitializeRiderBumps) -> Result<()> {
        self.rider_account.set_inner(Rider {
            wallet: self.rider.key(),
            id_status: true,            // TODO: Validation from KYC of Gov app to be added in future
            points: 0,
            penalties: 0,
            is_renting: false,
            bump: bumps.rider_account,
        });
        Ok(())
    }
}