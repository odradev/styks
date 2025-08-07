#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

use odra::prelude::*;

extern crate alloc;

pub mod heartbeat;
pub mod twap;

pub type PriceFeedId = String;
pub type Price = u64;
