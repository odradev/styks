#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate alloc;

use crate::price_feed::DurationSec;

pub mod error;
pub mod price_feed;
pub mod price_feed_manager;

pub fn minutes(n: u64) -> DurationSec {
    n * 60
}