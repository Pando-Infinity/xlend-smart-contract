use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};
use anchor_spl::token;
use crate::errors::ErrorCode;

pub fn verify_account_empty(account: &AccountInfo) -> ProgramResult {
    let notes_remaining = token::accessor::amount(account)?;

    if notes_remaining > 0 {
        msg!("the account is not empty");
        return Err(ErrorCode::AccountNotEmptyError.into());
    }

    Ok(())
}
