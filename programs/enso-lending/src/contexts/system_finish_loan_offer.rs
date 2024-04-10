// use anchor_lang::{prelude::*, solana_program::{system_instruction, program::invoke_signed}};
// use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};
// use crate::{
//   common::{
//     RepayOfferError, constant::LoanOfferStatus
//   }, SystemRepayLoadOfferNativeEvent, LOAN_OFFER_ACCOUNT_SEED, ENSO_SEED, states::loan_offer::LoanOfferAccount
// };

// #[derive(Accounts)]
// #[instruction(loan_offer_id: String)]
// pub struct SystemRepayLoadOfferNative<'info> {
//   #[account(mut)]
//   pub system: Signer<'info>,
//   #[account(
//     mut,
//     associated_token::mint = mint_asset,
//     associated_token::authority = system
//   )]
//   pub system_ata_asset: Account<'info, TokenAccount>,
//   #[account(
//     constraint = mint_asset.key() == loan_offer.lend_mint_token @ RepayOfferError::InvalidMintAsset,
//   )]
//   pub mint_asset: Account<'info, Mint>,
//   /// CHECK: This account is used to validate the wallet receive back lend amount
//   #[account(
//     constraint = lender.key() == loan_offer.lender @ RepayOfferError::InvalidLender
//   )]
//   pub lender: UncheckedAccount<'info>,
//   #[account(
//     mut,
//     associated_token::mint = mint_asset,
//     associated_token::authority = lender
//   )]
//   pub lender_ata_asset: Account<'info, TokenAccount>,
//   /// CHECK: This is the account used to receive back the collateral amount
//   #[account(mut)]
//   pub borrower: UncheckedAccount<'info>,
//   #[account(
//     mut,
//     constraint = loan_offer.status == LoanOfferStatus::Repay @ RepayOfferError::InvalidOfferStatus,
//     seeds = [
//       ENSO_SEED.as_ref(),
//       LOAN_OFFER_ACCOUNT_SEED.as_ref(),
//       borrower.key().as_ref(),
//       loan_offer_id.as_bytes(),
//       crate::ID.key().as_ref()
//     ],
//     bump = loan_offer.bump
//   )]
//   pub loan_offer: Account<'info, LoanOfferAccount>,
//   pub token_program: Program<'info, Token>,
//   pub system_program: Program<'info, System>,
// }

// impl<'info> SystemRepayLoadOfferNative<'info> {
//   pub fn system_repay_loan_offer(&mut self, collateral_amount: u64) -> Result<()>  {
//     // let interest_loan_amount = (self.loan_offer.interest * loan_amount as f64) as u64;
//     // let lender_fee_amount = (self.loan_offer.lender_fee_percent * loan_amount as f64) as u64;

//     // let total_repay_to_lender = loan_amount + waiting_interest + interest_loan_amount - lender_fee_amount;

//     self.transfer_asset_to_borrower(collateral_amount)?;
//     // self.transfer_asset_to_lender(loan_amount, total_repay_to_lender)?;
//     self.loan_offer.status = LoanOfferStatus::BorrowerPaid;

//     self.emit_event_system_repay_loan_offer(
//       String::from("system_repay_loan_offer"),
//       collateral_amount
//     )?;

//     Ok(())
//   }

//   fn transfer_asset_to_borrower(&mut self, collateral_amount: u64) -> Result<()> {
//     if self.borrower.key() != self.loan_offer.borrower.key() {
//       return Err(RepayOfferError::InvalidBorrower)?;
//     }

//     if collateral_amount != self.loan_offer.collateral_amount {
//       return Err(RepayOfferError::InvalidCollateralAmount)?;
//     }

//     self.process_transfer_collateral(collateral_amount)?;

//     Ok(())
//   }

//   fn process_transfer_collateral(&self, collateral_amount: u64) -> Result<()> {
//     let transfer_instruction = system_instruction::transfer(
//     &self.system.key(), 
//     &self.borrower.key(), 
//     collateral_amount
//     );
    
//     invoke_signed(
//       &transfer_instruction,
//       &[
//         self.system.to_account_info(),
//         self.borrower.to_account_info(),          
//         self.system_program.to_account_info()
//       ],
//       &[],  
//     )?;

//     Ok(())
//   }

//   fn transfer_asset_to_lender(&mut self, loan_amount: u64, total_repay_to_lender: u64) -> Result<()> {
//     if loan_amount != self.loan_offer.borrow_amount {
//       return Err(RepayOfferError::InvalidLendAmount)?;
//     }

//     if total_repay_to_lender > self.system_ata_asset.amount {
//       return Err(RepayOfferError::NotEnoughAmount)?;
//     }

//     self.process_transfer_lend_asset(total_repay_to_lender)?;

//     Ok(())
//   }

//   fn process_transfer_lend_asset(&mut self, total_repay: u64) -> Result<()> {
//     transfer_checked(
//         self.into_transfer_back_lend_asset_context(),
//         total_repay,
//         self.mint_asset.decimals,
//     )
//   }

//   fn into_transfer_back_lend_asset_context(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
//     let cpi_accounts = TransferChecked {
//         from: self.system_ata_asset.to_account_info(),
//         mint: self.mint_asset.to_account_info(),
//         to: self.lender_ata_asset.to_account_info(),
//         authority: self.system.to_account_info(),
//     };
//     CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
//   }

//   fn emit_event_system_repay_loan_offer(
//     &mut self,
//     label: String,
//     collateral_amount: u64
//   ) -> Result<()> {
//     emit!(SystemRepayLoadOfferNativeEvent {
//       system: self.system.key(),
//       lender: self.lender.key(),
//       borrower: self.borrower.key(),
//       interest: self.loan_offer.interest,
//       loan_amount: self.loan_offer.collateral_amount,
//       loan_offer_id: self.loan_offer.offer_id.clone(),
//       tier_id: self.loan_offer.tier_id.clone(),
//       collateral_amount,
//       status: self.loan_offer.status,
//     });

//     msg!(&label.clone());

//     Ok(())
//   }
// }