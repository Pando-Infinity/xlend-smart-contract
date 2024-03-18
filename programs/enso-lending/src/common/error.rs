use anchor_lang::error_code;

#[error_code]
pub enum SettingAccountError {
    #[msg("Invalid tier id")]
    InvalidTierId,   
    #[msg("Invalid owner account")]
    InvalidOwner, 
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
    #[msg("Invalid offer id")]
    InvalidOfferId,
    #[msg("Invalid receiver")]
    InvalidReceiver,
}

#[error_code]
pub enum LoanOfferError {
    #[msg("Invalid receiver")]
    InvalidReceiver,
    #[msg("Can not deposit collateral to loan offer that not available")]
    CanNotDepositCollateralToContractThatNotAvailable,
    #[msg("Can not take a loan because health ratio is not valid")]
    CanNotTakeALoanBecauseHealthRatioIsNotValid,
    #[msg("Invalid price feed account for collateral asset")]
    InvalidPriceFeedAccountForCollateralAsset,
    #[msg("Invalid price feed account for lend asset")]
    InvalidPriceFeedAccountForLendAsset,
    #[msg("Borrower does not have enough assets")]
    NotEnoughAmount,
    #[msg("Invalid mint asset")]
    InvalidMintAsset,
    #[msg("Loan offer status is invalid")]
    InvalidOfferStatus,
    #[msg("lend offer is not available")]
    LendOfferIsNotAvailable,
    #[msg("Health ratio limit")]
    HealthRatioLimit,
    #[msg("Duration loan offer invalid")]
    DurationLoanOfferInvalid,
    #[msg("Invalid operator system account")]
    InvalidSystem,
}

#[error_code]
pub enum RepayOfferError {
    #[msg("Invalid mint asset of loan offer")]
    InvalidMintAsset,
    #[msg("Borrower does not have enough assets")]
    NotEnoughAmount,
    #[msg("Loan offer is not available")]
    LoanOfferIsNotAvailable
}