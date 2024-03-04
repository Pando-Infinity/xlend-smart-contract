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
}