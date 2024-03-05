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
        amount: u64,
        duration: u64,
        lender_fee_percent: f64,
    ) -> Result<()> {
        ctx.accounts.init_setting_account(
            &ctx.bumps,
            tier_id.clone(),
            amount,
            duration,
            lender_fee_percent,
        )?;
        ctx.accounts
            .emit_init_setting_account_event(String::from("Emit event init setting account"))?;

        Ok(())
    }

    pub fn edit_setting_account(
        ctx: Context<EditSettingAccount>,
        _tier_id: String,
        amount: Option<u64>,
        duration: Option<u64>,
        lender_fee_percent: Option<f64>,
    ) -> Result<()> {
        ctx.accounts
            .edit_setting_account(amount, duration, lender_fee_percent)?;

        ctx.accounts
            .emit_event_edit_setting_account(String::from("edit_setting_account"))?;

        Ok(())
    }

    pub fn close_setting_account(ctx: Context<CloseSettingAccount>, tier_id: String) -> Result<()> {
        ctx.accounts.close_setting_account()?;

        ctx.accounts.emit_event_close_setting_account(
            String::from("edit_setting_account"),
            tier_id.clone(),
        )?;

        Ok(())
    }

    pub fn create_lend_offer(ctx: Context<CreateLendOffer>, offer_id: String, _tier_id: String, interest: f64) -> Result<()> {
        ctx.accounts.initialize_lend_offer(&ctx.bumps, offer_id, interest)?;
        ctx.accounts.deposit()?;
        ctx.accounts.emit_event_create_lend_offer(String::from("create_lend_offer"))?;

        Ok(())
    }

    pub fn edit_lend_offer(ctx: Context<EditLendOffer>, _offer_id: String, interest: f64) -> Result<()> {
        ctx.accounts.edit_lend_offer(interest)?;
        ctx.accounts.emit_event_edit_lend_offer(String::from("edit_lend_offer"))?;

        Ok(())
    }
}
