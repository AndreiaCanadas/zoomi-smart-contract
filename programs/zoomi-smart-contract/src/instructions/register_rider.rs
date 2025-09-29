use anchor_lang::prelude::*;
use crate::state::Rider;

#[derive(Accounts)]
pub struct RegisterRider<'info> {
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
impl<'info> RegisterRider<'info> {
    pub fn register_rider(&mut self, bumps: &RegisterRiderBumps) -> Result<()> {
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