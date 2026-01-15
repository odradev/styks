// --- Access Control Roles ---

use odra_modules::access::{Role, DEFAULT_ADMIN_ROLE};

#[derive(Debug)]
pub enum SupplierRole {
    Admin,
    ConfigManager,
}

impl SupplierRole {
    pub fn role_id(&self) -> Role {
        match self {
            SupplierRole::Admin => DEFAULT_ADMIN_ROLE,
            // start with 3, so it doesn't overlap with PriceFeed.
            SupplierRole::ConfigManager => [3u8; 32],
        }
    }
}
