use anchor_lang::prelude::*;
use pyth_sdk_solana::load_price_feed_from_account_info;

use crate::STALENESS_THRESHOLD;

pub fn convert_to_usd_price(price_feed_account_info: &AccountInfo) -> Result<u64> {
  let price_feed = load_price_feed_from_account_info(price_feed_account_info).unwrap();
  let current_timestamp = Clock::get()?.unix_timestamp;
  let current_price = price_feed.get_ema_price_no_older_than(current_timestamp, STALENESS_THRESHOLD).unwrap();
  let display_price = u64::try_from(current_price.price).unwrap() / 10u64.pow(u32::try_from(-current_price.expo).unwrap());

  Ok(display_price)
}