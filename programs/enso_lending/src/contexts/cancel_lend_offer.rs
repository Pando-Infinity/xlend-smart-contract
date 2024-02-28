use anchor_lang::__private::CLOSED_ACCOUNT_DISCRIMINATOR;
use anchor_lang::{prelude::*};
use std::io::{Cursor, Write};
use std::ops::DerefMut;
use crate::common::constant::CancelLendOfferEvent;
use crate::{LendOrderAccount, LendOrderStatus, LendOrderError};

#[derive(Accounts)]
#[instruction(order_id: String)]
pub struct CancelLendOffer<'info> {
  #[account(mut)]
  pub lender: Signer<'info>,
  #[account(
    mut,
    has_one = lender,
    constraint = lend_order.status == LendOrderStatus::Created @ LendOrderError::InvalidOrderStatus,
    constraint = lend_order.order_id == order_id @ LendOrderError::InvalidOrderId,
    seeds = [b"enso".as_ref(), lender.key().as_ref(), order_id.as_bytes()],
    bump
  )]
  pub lend_order: Account<'info, LendOrderAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
  pub token_program: AccountInfo<'info>,
  pub system_program: Program<'info, System>,
}

impl<'info> CancelLendOffer<'info> {
  pub fn close_lend_offer(&mut self) -> Result<()>  {
    let dest_starting_lamports = self.lender.lamports();
    let lend_order_account = self.lend_order.to_account_info();

    **self.lender.lamports.borrow_mut() = dest_starting_lamports
      .checked_add(lend_order_account.lamports())
      .unwrap();
    **lend_order_account.lamports.borrow_mut() = 0;

    let mut data = lend_order_account.try_borrow_mut_data()?;
    for byte in data.deref_mut().iter_mut() {
      *byte = 0;
    }

    let dst: &mut [u8] = &mut data;
    let mut cursor = Cursor::new(dst);
    cursor.write_all(&CLOSED_ACCOUNT_DISCRIMINATOR).unwrap();

    Ok(())
  }

  pub fn emit_event_cancel_lend_offer(&mut self, label: String, order_id: String) -> Result<()> {
    emit!(CancelLendOfferEvent {
      lender: self.lender.key(),
      order_id
    });
    
    msg!(&label.clone());
    
    Ok(())
  }
}