#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate alloc;

pub mod make_supplier;
pub mod staking;
#[deprecated(since = "2.0.0", note = "Use `make_supplier` instead")]
pub mod styks_blocky_supplier;
pub mod styks_price_feed;
pub mod supplier_error;
pub mod supplier_role;
pub mod token;
