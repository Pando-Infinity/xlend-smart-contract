use anchor_lang::prelude::*;

mod contexts;
use contexts::*;
mod states;
use states::*;
mod common;
use common::*;

declare_id!("CNkCiHgVyh6u1ifYb6YpK9bZAjD7oviJEsR5G1cMmLob");

#[program]
pub mod enso_lending {
    use super::*;

    pub fn init_setting_account(
        ctx: Context<InitSettingAccount>,
        tier_id: String,
        amount: f64,
        duration: u64,
    ) -> Result<()> {
        ctx.accounts.init_setting_account(amount, duration)?;
        ctx.accounts.emit_init_setting_account_event(
            tier_id,
            String::from("Emit event init setting account"),
        )?;

        Ok(())
    }
}
