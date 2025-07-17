#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
extern crate alloc;

pub mod error;
pub mod price_feed;
pub mod price_feed_manager;