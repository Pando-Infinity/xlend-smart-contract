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
    #[msg("Invalid Lend Amount")]
    InvalidLendAmount,
    #[msg("Not enough amount")]
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
    #[msg("Not enough amount")]
    NotEnoughAmount,
    #[msg("Invalid collateral mint asset")]
    InvalidCollateralMintAsset,
    #[msg("Invalid Lend mint asset")]
    InvalidLendMintAsset,
    #[msg("Loan offer status is invalid")]
    InvalidOfferStatus,
    #[msg("lend offer is not available")]
    LendOfferIsNotAvailable,
    #[msg("Health ratio limit")]
    HealthRatioLimit,
    #[msg("Loan offer expired")]
    LoanOfferExpired,
    #[msg("Invalid hot wallet account")]
    InvalidHotWallet,
    #[msg("Invalid operator system account")]
    InvalidSystem,
    #[msg("Invalid borrower")]
    InvalidBorrower,
    #[msg("Invalid loan offer")]
    InvalidLoanOffer,
    #[msg("Invalid borrow amount")]
    InvalidBorrowAmount,
    #[msg("Loan offer not available to withdraw")]
    NotAvailableToWithdraw
}

#[error_code]
pub enum LoanOfferCrossChainError {
    #[msg("Invalid receiver")]
    InvalidReceiver,
    #[msg("Invalid Lend mint asset")]
    InvalidLendMintAsset,
    #[msg("lend offer is not available")]
    LendOfferIsNotAvailable,
    #[msg("Invlaid collateral token symbol")]
    InvalidCollateralTokenSymbol,
    #[msg("Can not take a loan because health ratio is not valid")]
    CanNotTakeALoanBecauseHealthRatioIsNotValid,
    #[msg("Invalid wormhole system account")]
    InvalidSystem,
    #[msg("Invalid vaa")]
    InvalidVaa,
}

#[error_code]
pub enum RepayOfferError {
    #[msg("Invalid mint asset of loan offer")]
    InvalidMintAsset,
    #[msg("Not enough assets")]
    NotEnoughAmount,
    #[msg("Loan offer is not available")]
    LoanOfferIsNotAvailable,
    #[msg("Invalid lend amount")]
    InvalidLendAmount,
    #[msg("Loan offer not belong to borrower")]
    InvalidBorrower,
    #[msg("Invalid collateral amount")]
    InvalidCollateralAmount,
    #[msg("Invalid offer status")]
    InvalidOfferStatus,
    #[msg("Loan offer not belong to lender")]
    InvalidLender,
    #[msg("Invalid repay lender time")]
    TimeUnmetException
}

#[error_code]
pub enum LiquidateOfferError {
    #[msg("Loan offer not belong to lender")]
    InvalidLender,
     #[msg("Loan offer not belong to borrower")]
    InvalidBorrower,
    #[msg("Loan offer status is invalid")]
    InvalidOfferStatus,
    #[msg("Invalid lend amount")]
    InvalidLendAmount,
    #[msg("Not have enough amount of assets")]
    NotEnoughAmount,
    #[msg("Invalid mint asset")]
    InvalidMintAsset,
    #[msg("Invalid operator system account")]
    InvalidSystem,
}

#[error_code]
pub enum WormholeError {
    #[msg("Invalid wormhole system account")]
    InvalidSystem,
    #[msg("Invalid foreign emitter")]
    InvalidForeignEmitter,
    #[msg("Invalid message")]
    InvalidMessage,
}