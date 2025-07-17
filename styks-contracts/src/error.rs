pub use odra::prelude::*;

#[odra::odra_error]
pub enum PriceFeedError {
    TimestampInFuture,
    TimestampTooOld,
    NotAdminRole,
    NotPriceFeedManagerRole,
    NotPriceSupplierRole,
    PriceFeedAlreadyExists,
    PriceFeedNotFound,
}