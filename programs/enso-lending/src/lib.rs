use anchor_lang::prelude::*;

mod contexts;
use contexts::*;
mod states;
use states::*;
mod common;
use common::*;
mod utils;
use utils::*;

declare_id!("BderhzujHHQNjhCZGRjLWnN2XQ66q4EZnZx2p5WLJnBV");

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
            borrower_fee_percent,
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
        ctx.accounts.edit_setting_account(
            amount,
            duration,
            lender_fee_percent,
            borrower_fee_percent,
        )?;

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

    pub fn create_lend_offer(
        ctx: Context<CreateLendOffer>,
        offer_id: String,
        _tier_id: String,
        interest: f64,
    ) -> Result<()> {
        ctx.accounts
            .initialize_lend_offer(&ctx.bumps, offer_id, interest)?;
        ctx.accounts.deposit()?;
        ctx.accounts
            .emit_event_create_lend_offer(String::from("create_lend_offer"))?;

        Ok(())
    }

    pub fn edit_lend_offer(
        ctx: Context<EditLendOffer>,
        _offer_id: String,
        interest: f64,
    ) -> Result<()> {
        ctx.accounts.edit_lend_offer(interest)?;
        ctx.accounts
            .emit_event_edit_lend_offer(String::from("edit_lend_offer"))?;

        Ok(())
    }

    pub fn system_cancel_lend_offer(
        ctx: Context<SystemCancelLendOffer>,
        _offer_id: String,
        _tier_id: String,
        lend_amount: u64,
        waiting_interest: u64,
    ) -> Result<()> {
        ctx.accounts
            .system_cancel_lend_offer(lend_amount, waiting_interest)?;

        Ok(())
    }

    pub fn cancel_lend_offer(ctx: Context<CancelLendOffer>, _offer_id: String) -> Result<()> {
        ctx.accounts.cancel_lend_offer()?;

        ctx.accounts
            .emit_event_cancel_lend_offer(String::from("cancel_lend_offer"))?;

        Ok(())
    }

    pub fn create_loan_offer(
        ctx: Context<CreateLoanOffer>,
        offer_id: String,
        lend_offer_id: String,
        tier_id: String,
        collateral_amount: u64,
    ) -> Result<()> {
        ctx.accounts.initialize_loan_offer(
            &ctx.bumps,
            offer_id,
            lend_offer_id,
            tier_id,
            collateral_amount,
        )?;
        ctx.accounts
            .emit_event_create_loan_offer(String::from("create_loan_offer"))?;

        Ok(())
    }

    pub fn create_loan_offer_native(
        ctx: Context<CreateLoanOfferNative>,
        offer_id: String,
        lend_offer_id: String,
        tier_id: String,
        collateral_amount: u64,
    ) -> Result<()> {
        ctx.accounts.initialize_loan_offer(
            &ctx.bumps,
            offer_id,
            lend_offer_id,
            tier_id,
            collateral_amount,
        )?;
        ctx.accounts
            .emit_event_create_loan_offer(String::from("create_loan_offer_native"))?;

        Ok(())
    }

    pub fn system_update_loan_offer(
        ctx: Context<SystemUpdateLoanOffer>,
        _offer_id: String,
        _tier_id: String,
        borrow_amount: u64,
    ) -> Result<()> {
        ctx.accounts.system_update_loan_offer(borrow_amount)?;

        Ok(())
    }

    pub fn deposit_collateral_loan_offer(
        ctx: Context<DepositCollateralLoanOffer>,
        _offer_id: String,
        _tier_id: String,
        amount: u64,
    ) -> Result<()> {
        ctx.accounts.deposit_collateral_loan_offer(amount)?;
        ctx.accounts
            .emit_event_deposit_collateral_loan_offer(String::from(
                "deposit_collateral_loan_offer",
            ))?;

        Ok(())
    }

    pub fn deposit_collateral_loan_offer_native(
        ctx: Context<DepositCollateralLoanOfferNative>,
        _offer_id: String,
        _tier_id: String,
        amount: u64,
    ) -> Result<()> {
        ctx.accounts.deposit_collateral_loan_offer(amount)?;
        ctx.accounts
            .emit_event_deposit_collateral_loan_offer(String::from(
                "deposit_collateral_loan_offer_native",
            ))?;

        Ok(())
    }

    pub fn repay_loan_offer(ctx: Context<RepayLoanOffer>, _loan_offer_id: String) -> Result<()> {
        ctx.accounts.repay_loan_offer()?;

        Ok(())
    }

    pub fn system_repay_loan_offer(
        ctx: Context<SystemRepayLoadOfferNative>,
        _loan_offer_id: String,
        collateral_amount: u64,
    ) -> Result<()> {
        ctx.accounts.system_repay_loan_offer(collateral_amount)?;

        Ok(())
    }

    pub fn withdraw_collateral(
        ctx: Context<WithdrawCollateral>,
        loan_offer_id: String,
        withdraw_amount: u64,
    ) -> Result<()> {
        ctx.accounts.withdraw_collateral(loan_offer_id, withdraw_amount)?;
        
        Ok(())
    }

    pub fn system_transfer_collateral_request_withdraw(
        ctx: Context<SystemWithdrawNative>,
        loan_offer_id: String,
        withdraw_amount: u64,
    ) -> Result<()> {
        ctx.accounts.system_transfer_collateral_request_withdraw(loan_offer_id, withdraw_amount)?;
        
        Ok(())
    }

    pub fn start_liquidate_contract(
        ctx: Context<LiquidateCollateral>,
        _offer_id: String,
        liquidating_price: u64,
        liquidating_at: u64,
    ) -> Result<()> {
        ctx.accounts
            .start_liquidate_contract(liquidating_price, liquidating_at)?;
        ctx.accounts
            .emit_event_start_liquidate_contract(String::from("liquidating_collateral"))?;

        Ok(())
    }

    pub fn finish_liquidate_contract(
        ctx: Context<SystemLiquidateLoanOffer>,
        _loan_offer_id: String,
        collateral_swapped_amount: u64,
        liquidated_price: u64,
        liquidated_tx: String,
    ) -> Result<()> {
        ctx.accounts
            .system_liquidate_loan_offer(
              collateral_swapped_amount,
              liquidated_price, 
              liquidated_tx
            )?;

        Ok(())
    }

    pub fn system_finish_loan_offer(
      ctx: Context<SystemFinishLoanOffer>,
      _loan_offer_id: String,
      loan_amount: u64,
      interest_amount: u64,
    ) -> Result<()> {
        ctx.accounts
            .system_finish_loan_offer(
              loan_amount,
              interest_amount
            )?;

        Ok(())
    }

    pub fn system_revert_status(
      ctx: Context<SystemRevertStatus>,
      _offer_id: String
    ) -> Result<()> {
      ctx.accounts
        .system_revert_status()?;
      ctx.accounts
        .emit_event_revert_status(String::from("revert_status"))?;

      Ok(())
    }
}
