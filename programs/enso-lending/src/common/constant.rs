use anchor_lang::prelude::{borsh, AnchorDeserialize, AnchorSerialize, InitSpace};

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq, InitSpace, Debug)]
pub enum LendOfferStatus {
    Created,
    Canceling,
    Canceled,
    Loaned,
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq, InitSpace, Debug)]
pub enum LoanOfferStatus {
    Matched,
    FundTransferred,
    Repay,
    BorrowerPaid,
    Liquidating,
    Liquidated,
    Finished
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq, InitSpace, Debug)]
pub enum LoanOfferCrosschainStatus {
    WaitingCollateral,
    Matched,
    FundTransferred,
    Repay,
    BorrowerPaid,
    Liquidating,
    Liquidated,
    Finished
}

pub const ENSO_SEED: &[u8] = b"enso";
pub const SETTING_ACCOUNT_SEED: &[u8] = b"setting_account";
pub const LEND_OFFER_ACCOUNT_SEED: &[u8] = b"lend_offer";
pub const LOAN_OFFER_ACCOUNT_SEED: &[u8] = b"loan_offer";
pub const LOAN_OFFER_CROSSCHAIN_ACCOUNT_SEED: &[u8] = b"loan_offer_crosschain";

pub const OPERATE_SYSTEM_PUBKEY: &str = "opty8HWBKX3wW8c9qMPkmB4xnrCpMWWmQwqq7yGzmr4";
pub const HOT_WALLET_PUBKEY: &str = "Hot7zcvBTa3NybAnKrKtjcW1yJcoDWao39ZAoBn4mfPu";
pub const WORMHOLE_SYSTEM_PUBKEY: &str = "";

pub const SOL_USD_PRICE_FEED_ID: &str = "ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
pub const USDC_USD_PRICE_FEED_ID: &str = "eaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a";
pub const SUI_USD_PRICE_FEED_ID: &str = "23d7315113f5b1d3ba7a83604c44b94d79f4fd69af77f804fc7f920a6dc65744";

pub const SUI_USD_SYMBOL: &str = "Crypto.SUI/USD";
#[cfg(feature = "devnet")]
pub const MAXIMUM_AGE_PRICE_UPDATE: u64 = 100_000;
#[cfg(not(feature = "devnet"))]
pub const MAXIMUM_AGE_PRICE_UPDATE: u64 = 75;

pub const MIN_BORROW_HEALTH_RATIO: f64 = 1.1;

pub const NATIVE_MINT: &str = "So11111111111111111111111111111111111111112";