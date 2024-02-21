use anchor_lang::prelude::*;

mod contexts;
use contexts::*;
mod states;
use states::*;
mod common;
use common::*;

declare_id!("CNkCiHgVyh6u1ifYb6YpK9bZAjD7oviJEsR5G1cMmLob");

#[program]
pub mod smart_contract {
    use super::*;

    pub fn create_lend_order(
        ctx: Context<CreateLendOrder>,
        order_id: String,
        amount: u64,
        interest: f64,
        lender_fee: u64,
        duration: u64,
    ) -> Result<()> {
        ctx.accounts
            .initialize_lend_order(&ctx.bumps, order_id, amount, interest, lender_fee, duration)?;
        ctx.accounts.deposit(amount)
    }

    pub fn cancel_lend_offer(
        ctx: Context<CancelLendOffer>,
        order_id: String,
    ) -> Result<()> {
        ctx.accounts.close_lend_offer(order_id)
    }
}
