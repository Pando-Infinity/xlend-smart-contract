use anchor_lang::error_code;

#[error_code]
pub enum SettingAccountError {
    #[msg("Invalid tier id")]
    InvalidTierId,    
}

#[error_code]
pub enum LendOfferError {
    #[msg("Lender does not have enough assets")]
    NotEnoughAmount,
    #[msg("Invalid mint asset")]
    InvalidMintAsset,
    #[msg("Interest must be greater than 0")]
    InterestGreaterThanZero,
    #[msg("Lend offer is not initialized or not belong to lender")]
    InvalidLender,
    #[msg("Lend offer status is invalid")]
    InvalidOfferStatus,
}