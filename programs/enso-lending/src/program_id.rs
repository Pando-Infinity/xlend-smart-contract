use anchor_lang::prelude::Pubkey;
use solana_program::pubkey;

#[cfg(all(feature = "beta-test", feature = "dev-test"))]
compile_error!("'beta-test' and 'dev-test' features are mutually exclusive");

cfg_if::cfg_if!{
  if #[cfg(all(feature = "devnet", feature = "beta-test"))] {
    pub const PROGRAM_ID: Pubkey = pubkey!("BderhzujHHQNjhCZGRjLWnN2XQ66q4EZnZx2p5WLJnBV");
  } else if #[cfg(all(feature = "devnet", feature = "dev-test"))] {
    pub const PROGRAM_ID: Pubkey = pubkey!("wapafLYnVEXc8chECj84eVeGDj9UQTuBMdzmo23BNAM");
  } else {
    // Default use for localnet
    pub const PROGRAM_ID: Pubkey = pubkey!("G3LQL3DpD8Bd5q3ERGZwqAgkgyFKJTbt9ViCpu1hVd6o");
  }
}