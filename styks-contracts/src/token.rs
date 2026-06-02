use odra::{casper_types::U256, prelude::*};
use odra_modules::{access::Ownable, cep18_token::Cep18, security::Pauseable};

#[odra::module]
pub struct StyksToken {
    ownable: SubModule<Ownable>,
    pauseable: SubModule<Pauseable>,
    cep18: SubModule<Cep18>,
}

#[odra::module]
impl StyksToken {
    pub fn init(&mut self) {
        let deployer = self.env().caller();
        self.ownable.init(deployer);

        let decimals = 9;
        let initial_supply = U256::from(1_000_000).pow(U256::from(decimals));
        self.cep18.init(
            "Styks Token".to_string(),
            "STYKS".to_string(),
            decimals,
            initial_supply,
        );
    }

    pub fn mint(&mut self, recipient: &Address, amount: &U256) {
        self.pauseable.require_not_paused();
        let caller = self.env().caller();
        self.ownable.assert_owner(&caller);
        self.cep18.raw_mint(recipient, amount);
    }

    pub fn transfer(&mut self, recipient: &Address, amount: &U256) {
        self.pauseable.require_not_paused();
        self.cep18.transfer(recipient, amount);
    }

    pub fn transfer_from(&mut self, owner: &Address, recipient: &Address, amount: &U256) {
        self.pauseable.require_not_paused();
        self.cep18.transfer_from(owner, recipient, amount);
    }

    pub fn approve(&mut self, spender: &Address, amount: &U256) {
        self.pauseable.require_not_paused();
        self.cep18.approve(spender, amount);
    }

    pub fn increase_allowance(&mut self, spender: &Address, inc_by: &U256) {
        self.pauseable.require_not_paused();
        self.cep18.increase_allowance(spender, inc_by);
    }

    pub fn decrease_allowance(&mut self, spender: &Address, decr_by: &U256) {
        self.pauseable.require_not_paused();
        self.cep18.decrease_allowance(spender, decr_by);
    }

    delegate! {
        to self.cep18 {
            fn name(&self) -> String;
            fn symbol(&self) -> String;
            fn decimals(&self) -> u8;
            fn total_supply(&self) -> U256;
            fn balance_of(&self, owner: &Address) -> U256;
            fn allowance(&self, owner: &Address, spender: &Address) -> U256;
        }

        to self.pauseable {
            fn is_paused(&self) -> bool;
            fn pause(&mut self);
            fn unpause(&mut self);
        }
    }
}
