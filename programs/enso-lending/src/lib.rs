use anchor_lang::prelude::*;

mod contexts;
use contexts::*;
mod states;
use states::*;
mod common;
use common::*;
mod utils;
use utils::*;

declare_id!("4z4kmGW4AcmBoyeGobKDXXTRizSSuzXLroX6zjkyeYA1");

#[program]
pub mod enso_lending {

    use super::*;

    pub fn init_setting_account(
        ctx: Context<InitSettingAccount>,
        tier_id: String,
        amount: u64,
        duration: u64,
        lender_fee_percent: f64,
        borrower_fee_percent: f64,
    ) -> Result<()> {
        ctx.accounts.init_setting_account(
            &ctx.bumps,
            tier_id.clone(),
            amount,
            duration,
            lender_fee_percent,
            borrower_fee_percent
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
        borrower_fee_percent: Option<f64>,
    ) -> Result<()> {
        ctx.accounts
            .edit_setting_account(amount, duration, lender_fee_percent, borrower_fee_percent)?;

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

    pub fn cancel_lend_offer(ctx: Context<CancelLendOffer>, offer_id: String) -> Result<()> {
        ctx.accounts.cancel_lend_offer()?;

        ctx.accounts.emit_event_cancel_lend_offer(
            String::from("cancel_lend_offer"),
            offer_id.clone(),
        )?;

        Ok(())
    }

    pub fn create_loan_offer(
        ctx: Context<CreateLoanOffer>, 
        offer_id: String, 
        lend_offer_id: String, 
        tier_id: String, 
        collateral_amount: u64
    ) -> Result<()> {
        ctx.accounts.initialize_loan_offer(&ctx.bumps, offer_id, lend_offer_id, tier_id, collateral_amount)?;
        ctx.accounts.emit_event_create_loan_offer(String::from("create_loan_offer"))?;

        Ok(())
    }

    pub fn deposit_collateral_loan_offer(
        ctx: Context<DepositCollateralLoanOffer>,
        _offer_id: String,
        _tier_id: String,
        amount: u64
    ) -> Result<()> {
        ctx.accounts.deposit_collateral_loan_offer(amount)?;
        ctx.accounts.emit_event_deposit_collateral_loan_offer(String::from("deposit_collateral_loan_offer"))?;

        Ok(())
    }

    pub fn repay_loan_offer(ctx: Context<RepayLoanOffer>, _loan_offer_id: String) -> Result<()> {
        ctx.accounts.repay_loan_offer()?;

        Ok(())
    }

    pub fn withdraw_collateral(
        ctx: Context<WithdrawCollateral>,
        loan_offer_id: String,
        withdraw_amount: u64
    ) -> Result<()> {
        ctx.accounts.withdraw_collateral(withdraw_amount)?;
        ctx.accounts.emit_event_withdraw_collateral(String::from("withdraw_collateral"), loan_offer_id, withdraw_amount)?;

        Ok(())
    }
    pub fn liquidating_collateral(
        ctx: Context<LiquidatingCollateral>,
        liquidating_price: u64,
        liquidating_at: u64
    ) -> Result<()> {
        ctx.accounts.liquidating_collateral(liquidating_price, liquidating_at)?;
        ctx.accounts.emit_event_liquidating_collateral(String::from("liquidating_collateral"))?;

        Ok(())
    }

    pub fn liquidated_collateral(
        ctx: Context<LiquidatedCollateral>,
        liquidated_price: u64,
        liquidated_tx: String,
    ) -> Result<()> {
        ctx.accounts.liquidated_collateral(liquidated_price, liquidated_tx)?;
        ctx.accounts.emit_event_liquidated_collateral(String::from("Liquidated_collateral"))?;

        Ok(())
    }

}
