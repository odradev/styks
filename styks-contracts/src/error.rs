pub use odra::prelude::*;

#[odra::odra_error]
pub enum PriceFeedError {
    TimestampInFuture = 25000,
    TimestampTooOld = 25001,
    NotAdminRole = 25002,
    NotPriceFeedManagerRole = 25003,
    NotPriceSupplierRole = 25004,
    PriceFeedAlreadyExists = 25005,
    PriceFeedNotFound = 25006,
}