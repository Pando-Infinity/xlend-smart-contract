use anchor_lang::__private::CLOSED_ACCOUNT_DISCRIMINATOR;
use anchor_lang::{prelude::*};
use std::io::{Cursor, Write};
use std::ops::DerefMut;
use crate::common::{CloseLendOfferEvent, LendOfferStatus, LendOfferError};
use crate::states::lend_offer::LendOfferAccount;

#[derive(Accounts)]
#[instruction(offer_id: String)]
pub struct CloseLendOffer<'info> {
#[account(mut)]
  pub lender: Signer<'info>,
  #[account(
    mut,
    constraint = lend_offer.status == LendOfferStatus::Created @ LendOfferError::InvalidOfferStatus,
    seeds = [
        b"enso".as_ref(), 
        b"lend_offer".as_ref(), 
        lender.key().as_ref(), 
        offer_id.as_bytes(),
        crate::ID.key().as_ref(), 
    ],
    bump = lend_offer.bump
  )]
  pub lend_offer: Account<'info, LendOfferAccount>,
}

impl<'info> CloseLendOffer<'info> {
  pub fn close_lend_offer(&mut self) -> Result<()>  {
    let dest_starting_lamports = self.lender.lamports();
    let lend_offer_account = self.lend_offer.to_account_info();

    **self.lender.lamports.borrow_mut() = dest_starting_lamports
      .checked_add(lend_offer_account.lamports())
      .unwrap();
    **lend_offer_account.lamports.borrow_mut() = 0;

    let mut data = lend_offer_account.try_borrow_mut_data()?;
    for byte in data.deref_mut().iter_mut() {
      *byte = 0;
    }

    let dst: &mut [u8] = &mut data;
    let mut cursor = Cursor::new(dst);
    cursor.write_all(&CLOSED_ACCOUNT_DISCRIMINATOR).unwrap();

    Ok(())
  }

  pub fn emit_event_close_lend_offer(&mut self, label: String, offer_id: String) -> Result<()> {
    emit!(CloseLendOfferEvent {
      lender: self.lender.key(),
      offer_id
    });

    msg!(&label.clone());

    Ok(())
  }
}