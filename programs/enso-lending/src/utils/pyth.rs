use anchor_lang::prelude::*;
use pyth_sdk_solana::*;

use crate::STALENESS_THRESHOLD;

pub fn convert_to_usd_price(price_feed_account_info: &AccountInfo, amount: f64) -> Result<f64> {
    let price_feed = load_price_feed_from_account_info(price_feed_account_info).unwrap();
    let current_timestamp = Clock::get()?.unix_timestamp;
    let current_price = price_feed
        .get_ema_price_no_older_than(current_timestamp, STALENESS_THRESHOLD)
        .unwrap();
    let display_price = current_price.price as f64 / 10f64.powf(f64::try_from(-current_price.expo).unwrap());

    Ok(display_price * amount)
}
