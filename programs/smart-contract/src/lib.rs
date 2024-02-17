use anchor_lang::prelude::*;

mod contexts;
use contexts::*;
mod states;
use states::*;
mod common;
use common::*;
mod utils;
mod errors;


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
    ) -> Result<()> {
        ctx.accounts.initialize_lend_order(
            &ctx.bumps,
            order_id,
            amount,
            interest,
            lender_fee,
        )?;
        let _ = ctx.accounts.deposit(amount);
        ctx.accounts.emit_event_create_lend_order("Lend order created.".to_string())
    }
}
