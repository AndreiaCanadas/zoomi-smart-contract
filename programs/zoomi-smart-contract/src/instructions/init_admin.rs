use anchor_lang::prelude::*;
use crate::state::admin::Admin;

#[derive(Accounts)]
pub struct InitAdmin<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init, 
        payer = admin, 
        space = 8 + Admin::INIT_SPACE,
        seeds = [b"zoomi", admin.key().as_ref()],
        bump,
    )]
    pub admin_account: Account<'info, Admin>,
    #[account(
        seeds = [b"treasury", admin_account.key().as_ref()],
        bump,
    )]
    pub treasury: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}
impl<'info> InitAdmin<'info> {
    pub fn init_admin(&mut self, bumps: &InitAdminBumps) -> Result<()> {
        self.admin_account.set_inner(Admin { 
            admin: self.admin.key(),
            fee: 5,
            treasury_bump: bumps.treasury, 
            bump: bumps.admin_account,
        });
        Ok(())
    }
}