//! # StakingRewards
//!
//! Registry, stake custody, epoch reward accounting, external bounties and
//! slashing for Styks 2.0. See `Stysk2.0.md` (Resolved design) for the
//! rationale behind every rule encoded here.
//!
//! Custody is *transfer-in*: stake / bounty / pool funding all pull STYKS into
//! this contract via `transfer_from` (after a CEP-18 `approve`). Slashing and
//! payouts are then plain balance writes on the contract's own records plus a
//! `transfer` out. The contract therefore needs the STYKS token address, which
//! is supplied to `init`.
//!
//! ## v1 deviations from the design doc (intentional, documented)
//! - **Registered providers list is append-only.** Odra's `List` has no cheap
//!   arbitrary removal. A provider who fully exits (`active + pending == 0`)
//!   stays in the list but contributes zero eligible stake, so iteration in
//!   `finalize` / `finalize_bounty` naturally filters them out. Re-entry never
//!   pushes a duplicate (guarded by `listed`).
//! - **Bounty "reported in `[start, end]`" is approximated** by "met the epoch
//!   uptime gate for every epoch the window touches AND most-recent report is
//!   at/after `start`". We track per-epoch report counts and a single
//!   last-report timestamp, not per-report timestamps. See `is_bounty_eligible`.

use odra::{casper_types::U256, prelude::*, ContractRef};
use odra_modules::{
    access::{AccessControl, Role, DEFAULT_ADMIN_ROLE},
    security::Pauseable,
};

use crate::token::StyksTokenContractRef;

// --- Hardcoded protocol constants ---

/// Epoch length in seconds (1 day). Epoch boundaries are wall-clock UTC.
const EPOCH_LENGTH: u64 = 86_400;
/// Cooldown between `request_unstake` and `withdraw` (7 days).
const UNSTAKE_COOLDOWN: u64 = 7 * EPOCH_LENGTH;
/// Lower bound for a bounty time delta (no sub-minute bounties).
const BOUNTY_MIN_TIMEDELTA: u64 = 60;
/// Upper bound for a bounty time delta (no multi-epoch bounties in v1).
const BOUNTY_MAX_TIMEDELTA: u64 = EPOCH_LENGTH;

/// 10^9 — STYKS has 9 decimals.
const ONE_STYKS: u64 = 1_000_000_000;
/// Default `min_stake`: 10,000 STYKS.
const DEFAULT_MIN_STAKE: u64 = 10_000;
/// Default `reward_per_epoch`: 100 STYKS.
const DEFAULT_REWARD_PER_EPOCH: u64 = 100;
/// Default uptime threshold: 24 reports/day.
const DEFAULT_UPTIME_THRESHOLD: u32 = 24;

fn styks(whole: u64) -> U256 {
    U256::from(whole) * U256::from(ONE_STYKS)
}

// --- Errors ---

#[odra::odra_error]
pub enum StakingError {
    // Staking errors.
    TokenNotSet = 47000,
    ZeroAmount = 47001,
    InsufficientStake = 47002,
    UnstakeBelowFloor = 47003,
    PendingUnstakeExists = 47004,
    NoPendingUnstake = 47005,
    UnstakeStillLocked = 47006,

    // Role errors.
    NotAdmin = 47100,
    NotPauser = 47101,
    NotReporter = 47102,

    // Epoch / reward errors.
    EpochNotEnded = 47200,
    EpochAlreadyFinalized = 47201,
    EpochNotFinalized = 47202,
    AlreadyClaimed = 47203,
    NotEligible = 47204,

    // Slashing errors.
    NothingToSlash = 47300,
    SlashExceedsCap = 47301,
    AlreadySlashedThisEpoch = 47302,

    // Bounty errors.
    InvalidTimedelta = 47400,
    BountyNotFound = 47401,
    BountyNotEnded = 47402,
    BountyAlreadyFinalized = 47403,
    BountyNotFinalized = 47404,
}

// --- Access control roles ---

#[derive(Debug)]
pub enum StakingRole {
    /// Config, slashing, unpause. The deployer key in v1.
    Admin,
    /// Can pause (but not unpause). Asymmetry is deliberate.
    Pauser,
    /// Granted to the supplier(s); the only callers of `record_report`.
    Reporter,
}

impl StakingRole {
    pub fn role_id(&self) -> Role {
        match self {
            StakingRole::Admin => DEFAULT_ADMIN_ROLE,
            StakingRole::Pauser => [10u8; 32],
            StakingRole::Reporter => [11u8; 32],
        }
    }
}

// --- Stored types ---

/// A provider's single in-flight unstake request.
#[odra::odra_type]
pub struct PendingUnstake {
    pub amount: U256,
    pub unlock_time: u64,
}

/// A time-bounded external bounty funded by any STYKS holder.
#[odra::odra_type]
pub struct Bounty {
    pub depositor: Address,
    pub amount: U256,
    pub start: u64,
    pub end: u64,
    pub start_epoch: u64,
    pub end_epoch: u64,
}

// --- Events ---

#[odra::event]
pub struct Staked {
    pub provider: Address,
    pub amount: U256,
    pub active_stake: U256,
}

#[odra::event]
pub struct UnstakeRequested {
    pub provider: Address,
    pub amount: U256,
    pub unlock_time: u64,
}

#[odra::event]
pub struct Withdrawn {
    pub provider: Address,
    pub amount: U256,
}

#[odra::event]
pub struct ReportRecorded {
    pub provider: Address,
    pub epoch: u64,
    pub count: u32,
}

#[odra::event]
pub struct EpochFinalized {
    pub epoch: u64,
    pub total_eligible_stake: U256,
    pub budget: U256,
}

#[odra::event]
pub struct RewardClaimed {
    pub provider: Address,
    pub epoch: u64,
    pub amount: U256,
}

#[odra::event]
pub struct LowPool {
    pub epoch: u64,
    pub available: U256,
    pub required: U256,
}

#[odra::event]
pub struct Slashed {
    pub provider: Address,
    pub amount: U256,
    pub epoch: u64,
    pub reason_hash: [u8; 32],
}

#[odra::event]
pub struct PoolFunded {
    pub funder: Address,
    pub amount: U256,
}

#[odra::event]
pub struct BountyDeposited {
    pub id: u64,
    pub depositor: Address,
    pub amount: U256,
    pub start: u64,
    pub end: u64,
}

#[odra::event]
pub struct BountyFinalized {
    pub id: u64,
    pub total_eligible_stake: U256,
}

#[odra::event]
pub struct BountyStranded {
    pub id: u64,
    pub amount: U256,
}

#[odra::event]
pub struct BountyClaimed {
    pub id: u64,
    pub provider: Address,
    pub amount: U256,
}

// --- Contract ---

#[odra::module]
pub struct StakingRewards {
    access_control: SubModule<AccessControl>,
    pauseable: SubModule<Pauseable>,

    /// STYKS token contract address (custody + payouts).
    token: External<StyksTokenContractRef>,
    /// `deploy_time / EPOCH_LENGTH`; the genesis epoch has a zero uptime gate.
    genesis_epoch: Var<u64>,

    // Owner-configurable parameters.
    min_stake: Var<U256>,
    // Uptime threshold in reports/epoch for eligibility. Set to 0 for no uptime
    uptime_threshold: Var<u32>,
    reward_per_epoch: Var<U256>,

    // Staking state.
    active_stake: Mapping<Address, U256>,
    pending_unstake: Mapping<Address, Option<PendingUnstake>>,
    registered_providers: List<Address>,
    listed: Mapping<Address, bool>,

    /// Claimable reward funds held by the contract (separate from staked /
    /// bounty balances). Grows via `fund_pool`, slashing and stranded bounties.
    reward_pool: Var<U256>,

    // Per-epoch accounting.
    epoch_threshold: Mapping<u64, u32>,
    epoch_stake_snapshot: Mapping<(u64, Address), U256>,
    epoch_reports: Mapping<(u64, Address), u32>,
    epoch_finalized: Mapping<u64, bool>,
    epoch_total_eligible_stake: Mapping<u64, U256>,
    epoch_budget: Mapping<u64, U256>,
    epoch_claimed: Mapping<(u64, Address), bool>,

    /// Last epoch in which a provider was slashed (at most one slash/epoch).
    last_slash_epoch: Mapping<Address, u64>,
    /// Most recent report timestamp per provider (bounty eligibility).
    last_report_time: Mapping<Address, u64>,

    // Bounty state.
    next_bounty_id: Var<u64>,
    bounties: Mapping<u64, Bounty>,
    bounty_finalized: Mapping<u64, bool>,
    bounty_total_eligible_stake: Mapping<u64, U256>,
    bounty_claimed: Mapping<(u64, Address), bool>,
}

#[odra::module]
impl StakingRewards {
    /// Initialize the contract. `token` is the STYKS token address used for all
    /// custody (`transfer_from`) and payout (`transfer`) calls.
    pub fn init(&mut self, token: Address) {
        let deployer = self.env().caller();
        self.access_control
            .unchecked_grant_role(&StakingRole::Admin.role_id(), &deployer);
        self.access_control
            .unchecked_grant_role(&StakingRole::Pauser.role_id(), &deployer);

        self.token.set(token);

        let now = self.env().get_block_time_secs();
        self.genesis_epoch.set(now / EPOCH_LENGTH);

        self.min_stake.set(styks(DEFAULT_MIN_STAKE));
        self.uptime_threshold.set(DEFAULT_UPTIME_THRESHOLD);
        self.reward_per_epoch.set(styks(DEFAULT_REWARD_PER_EPOCH));
        self.reward_pool.set(U256::zero());
        self.next_bounty_id.set(0);
    }

    delegate! {
        to self.access_control {
            fn has_role(&self, role: &Role, address: &Address) -> bool;
            fn grant_role(&mut self, role: &Role, address: &Address);
            fn revoke_role(&mut self, role: &Role, address: &Address);
            fn get_role_admin(&self, role: &Role) -> Role;
            fn renounce_role(&mut self, role: &Role, address: &Address);
        }

        to self.pauseable {
            fn is_paused(&self) -> bool;
        }
    }

    // --- Pausability ---

    /// Pause ingress. Callable by `Pauser` or `Admin`.
    pub fn pause(&mut self) {
        let caller = self.env().caller();
        if !self.has_role(&StakingRole::Pauser.role_id(), &caller)
            && !self.has_role(&StakingRole::Admin.role_id(), &caller)
        {
            self.env().revert(StakingError::NotPauser);
        }
        self.pauseable.pause();
    }

    /// Resume ingress. `Admin` only — releasing the brakes is deliberate.
    pub fn unpause(&mut self) {
        self.assert_admin();
        self.pauseable.unpause();
    }

    // --- Configuration (Admin) ---

    pub fn set_min_stake(&mut self, min_stake: U256) {
        self.assert_admin();
        self.min_stake.set(min_stake);
    }

    pub fn set_uptime_threshold(&mut self, threshold: u32) {
        self.assert_admin();
        self.uptime_threshold.set(threshold);
    }

    pub fn set_reward_per_epoch(&mut self, reward: U256) {
        self.assert_admin();
        self.reward_per_epoch.set(reward);
    }

    // --- Reward pool funding ---

    /// Pull `amount` STYKS from the caller into the reward pool. The deploy
    /// recipe funds the initial pool this way (after an `approve`) rather than a
    /// raw transfer, so the on-chain pool accounting stays correct.
    pub fn fund_pool(&mut self, amount: U256) {
        if amount.is_zero() {
            self.env().revert(StakingError::ZeroAmount);
        }
        self.transfer_to_contract(&amount);
        self.reward_pool.add(amount);

        let funder = self.env().caller();
        self.env().emit_event(PoolFunded { funder, amount });
    }

    // --- Staking ---

    /// Stake `amount` STYKS (additive over existing stake). Requires a prior
    /// CEP-18 `approve` of this contract for `amount`.
    pub fn stake(&mut self, amount: U256) {
        self.pauseable.require_not_paused();
        if amount.is_zero() {
            self.env().revert(StakingError::ZeroAmount);
        }

        self.transfer_to_contract(&amount);

        let caller = self.env().caller();
        let active = self.active_stake.get_or_default(&caller) + amount;
        self.active_stake.add(&caller, amount);

        // Register on the first stake that brings the provider to >= min_stake.
        if active >= self.min_stake.get_or_default() && !self.listed.get_or_default(&caller) {
            self.registered_providers.push(caller);
            self.listed.set(&caller, true);
        }

        self.env().emit_event(Staked {
            provider: caller,
            amount,
            active_stake: active,
        });
    }

    /// Move `amount` from `active` to a single `pending` slot with a 7-day
    /// cooldown. Reverts if it would leave active in `(0, min_stake)` — the
    /// provider must keep at least `min_stake` or exit fully to 0.
    pub fn request_unstake(&mut self, amount: U256) {
        self.pauseable.require_not_paused();
        let caller = self.env().caller();

        if self.pending_unstake.get_or_default(&caller).is_some() {
            self.env().revert(StakingError::PendingUnstakeExists);
        }

        let active = self.active_stake.get_or_default(&caller);
        if amount.is_zero() || amount > active {
            self.env().revert(StakingError::InsufficientStake);
        }

        let remaining = active - amount;
        let min = self.min_stake.get_or_default();
        if remaining > U256::zero() && remaining < min {
            self.env().revert(StakingError::UnstakeBelowFloor);
        }

        self.active_stake.set(&caller, remaining);
        let unlock_time = self.env().get_block_time_secs() + UNSTAKE_COOLDOWN;
        self.pending_unstake.set(
            &caller,
            Some(PendingUnstake {
                amount,
                unlock_time,
            }),
        );

        self.env().emit_event(UnstakeRequested {
            provider: caller,
            amount,
            unlock_time,
        });
    }

    /// Release a matured pending unstake. Always callable, even when paused —
    /// user exits never get trapped.
    pub fn withdraw(&mut self) {
        let caller = self.env().caller();
        let pending = self
            .pending_unstake
            .get_or_default(&caller)
            .unwrap_or_revert_with(&self.env(), StakingError::NoPendingUnstake);

        if self.env().get_block_time_secs() < pending.unlock_time {
            self.env().revert(StakingError::UnstakeStillLocked);
        }

        self.pending_unstake.set(&caller, None);
        self.token.transfer(&caller, &pending.amount);

        self.env().emit_event(Withdrawn {
            provider: caller,
            amount: pending.amount,
        });
    }

    // --- Reporting ---

    /// Credit `reporter` with a report in the current epoch. Gated by the
    /// `Reporter` role on the *caller* (the supplier). Uses the on-chain clock
    /// for epoch math — no timestamp parameter.
    pub fn record_report(&mut self, reporter: Address) {
        self.pauseable.require_not_paused();
        self.assert_reporter();

        let now = self.env().get_block_time_secs();
        let epoch = now / EPOCH_LENGTH;

        // Snapshot the uptime threshold for this epoch on its first report
        // (forward-only: config changes affect future epochs only).
        if self.epoch_threshold.get(&epoch).is_none() {
            let threshold = if epoch == self.genesis_epoch.get_or_default() {
                0
            } else {
                self.uptime_threshold.get_or_default()
            };
            self.epoch_threshold.set(&epoch, threshold);
        }

        // Lazily snapshot the reporter's active stake for this epoch.
        let snap_key = (epoch, reporter);
        if self.epoch_stake_snapshot.get(&snap_key).is_none() {
            self.epoch_stake_snapshot
                .set(&snap_key, self.active_stake.get_or_default(&reporter));
        }

        let count = self.epoch_reports.get_or_default(&snap_key) + 1;
        self.epoch_reports.set(&snap_key, count);
        self.last_report_time.set(&reporter, now);

        self.env().emit_event(ReportRecorded {
            provider: reporter,
            epoch,
            count,
        });
    }

    // --- Finalization & claim ---

    /// Tally eligible stake for a finished epoch and reserve its budget from the
    /// pool. Permissionless, single-shot, callable even when paused.
    pub fn finalize(&mut self, epoch: u64) {
        let current_epoch = self.env().get_block_time_secs() / EPOCH_LENGTH;
        if epoch >= current_epoch {
            self.env().revert(StakingError::EpochNotEnded);
        }
        if self.epoch_finalized.get_or_default(&epoch) {
            self.env().revert(StakingError::EpochAlreadyFinalized);
        }

        let threshold = self.epoch_threshold.get_or_default(&epoch);

        let mut total_eligible = U256::zero();
        let len = self.registered_providers.len();
        for i in 0..len {
            if let Some(p) = self.registered_providers.get(i) {
                let reports = self.epoch_reports.get_or_default(&(epoch, p));
                if reports >= threshold && reports > 0 {
                    total_eligible += self.epoch_stake_snapshot.get_or_default(&(epoch, p));
                }
            }
        }
        self.epoch_total_eligible_stake.set(&epoch, total_eligible);

        // Reserve the per-epoch budget from the pool, or pay 0 if underfunded.
        let budget = self.reward_per_epoch.get_or_default();
        let pool = self.reward_pool.get_or_default();
        let reserved = if total_eligible.is_zero() {
            U256::zero()
        } else if pool >= budget {
            self.reward_pool.set(pool - budget);
            budget
        } else {
            self.env().emit_event(LowPool {
                epoch,
                available: pool,
                required: budget,
            });
            U256::zero()
        };
        self.epoch_budget.set(&epoch, reserved);
        self.epoch_finalized.set(&epoch, true);

        self.env().emit_event(EpochFinalized {
            epoch,
            total_eligible_stake: total_eligible,
            budget: reserved,
        });
    }

    /// Claim the caller's stake-weighted share of a finalized epoch. O(1).
    /// Always callable, even when paused.
    pub fn claim(&mut self, epoch: u64) {
        let caller = self.env().caller();
        if !self.epoch_finalized.get_or_default(&epoch) {
            self.env().revert(StakingError::EpochNotFinalized);
        }
        if self.epoch_claimed.get_or_default(&(epoch, caller)) {
            self.env().revert(StakingError::AlreadyClaimed);
        }

        let threshold = self.epoch_threshold.get_or_default(&epoch);
        let reports = self.epoch_reports.get_or_default(&(epoch, caller));
        if reports < threshold || reports == 0 {
            self.env().revert(StakingError::NotEligible);
        }

        let total = self.epoch_total_eligible_stake.get_or_default(&epoch);
        if total.is_zero() {
            self.env().revert(StakingError::NotEligible);
        }

        let budget = self.epoch_budget.get_or_default(&epoch);
        let snapshot = self.epoch_stake_snapshot.get_or_default(&(epoch, caller));
        let share = budget * snapshot / total;

        self.epoch_claimed.set(&(epoch, caller), true);
        if !share.is_zero() {
            self.token.transfer(&caller, &share);
        }

        self.env().emit_event(RewardClaimed {
            provider: caller,
            epoch,
            amount: share,
        });
    }

    // --- Slashing (Admin) ---

    /// Slash up to 50% of a provider's `active + pending` stake into the reward
    /// pool. At most one slash per provider per epoch. Always callable, even
    /// when paused — slashing may be the response to whatever triggered a pause.
    pub fn slash(&mut self, provider: Address, amount: U256, reason_hash: [u8; 32]) {
        self.assert_admin();

        let active = self.active_stake.get_or_default(&provider);
        let pending = self.pending_unstake.get_or_default(&provider);
        let pending_amount = pending.as_ref().map(|p| p.amount).unwrap_or_default();
        let total = active + pending_amount;

        if amount.is_zero() || total.is_zero() {
            self.env().revert(StakingError::NothingToSlash);
        }
        // amount <= 0.5 * total  <=>  2 * amount <= total
        if amount * U256::from(2) > total {
            self.env().revert(StakingError::SlashExceedsCap);
        }

        let epoch = self.env().get_block_time_secs() / EPOCH_LENGTH;
        if self.last_slash_epoch.get(&provider) == Some(epoch) {
            self.env().revert(StakingError::AlreadySlashedThisEpoch);
        }
        self.last_slash_epoch.set(&provider, epoch);

        // Active stake slashed first; pending only if amount exceeds active.
        if amount <= active {
            self.active_stake.set(&provider, active - amount);
        } else {
            let from_pending = amount - active;
            self.active_stake.set(&provider, U256::zero());
            let mut p = pending.unwrap_or_revert_with(&self.env(), StakingError::NothingToSlash);
            p.amount -= from_pending;
            self.pending_unstake.set(&provider, Some(p));
        }

        // Recycle slashed STYKS to honest providers rather than burning them.
        self.reward_pool.add(amount);

        self.env().emit_event(Slashed {
            provider,
            amount,
            epoch,
            reason_hash,
        });
    }

    // --- External bounties ---

    /// Fund a time-bounded bonus. Requires a prior `approve`. Returns the bounty
    /// id. `timedelta` must be within `[60s, 86_400s]`.
    pub fn deposit_with_timedelta(&mut self, amount: U256, timedelta: u64) -> u64 {
        self.pauseable.require_not_paused();
        if amount.is_zero() {
            self.env().revert(StakingError::ZeroAmount);
        }
        if !(BOUNTY_MIN_TIMEDELTA..=BOUNTY_MAX_TIMEDELTA).contains(&timedelta) {
            self.env().revert(StakingError::InvalidTimedelta);
        }

        let caller = self.env().caller();
        let self_address = self.env().self_address();
        self.token.transfer_from(&caller, &self_address, &amount);

        let now = self.env().get_block_time_secs();
        let end = now + timedelta;
        let id = self.next_bounty_id.get_or_default();
        self.next_bounty_id.set(id + 1);

        self.bounties.set(
            &id,
            Bounty {
                depositor: caller,
                amount,
                start: now,
                end,
                start_epoch: now / EPOCH_LENGTH,
                end_epoch: end / EPOCH_LENGTH,
            },
        );

        self.env().emit_event(BountyDeposited {
            id,
            depositor: caller,
            amount,
            start: now,
            end,
        });
        id
    }

    /// Tally eligible stake for a finished bounty. Permissionless, single-shot.
    /// If nobody is eligible the amount rolls into the reward pool (not
    /// refundable to the depositor).
    pub fn finalize_bounty(&mut self, id: u64) {
        let bounty = self
            .bounties
            .get(&id)
            .unwrap_or_revert_with(&self.env(), StakingError::BountyNotFound);

        if self.env().get_block_time_secs() < bounty.end {
            self.env().revert(StakingError::BountyNotEnded);
        }
        if self.bounty_finalized.get_or_default(&id) {
            self.env().revert(StakingError::BountyAlreadyFinalized);
        }

        let mut total_eligible = U256::zero();
        let len = self.registered_providers.len();
        for i in 0..len {
            if let Some(p) = self.registered_providers.get(i) {
                if self.is_bounty_eligible(&bounty, &p) {
                    total_eligible += self
                        .epoch_stake_snapshot
                        .get_or_default(&(bounty.start_epoch, p));
                }
            }
        }
        self.bounty_total_eligible_stake.set(&id, total_eligible);
        self.bounty_finalized.set(&id, true);

        if total_eligible.is_zero() {
            self.reward_pool.add(bounty.amount);
            self.env().emit_event(BountyStranded {
                id,
                amount: bounty.amount,
            });
        } else {
            self.env().emit_event(BountyFinalized {
                id,
                total_eligible_stake: total_eligible,
            });
        }
    }

    /// Claim the caller's stake-weighted share of a finalized bounty. Always
    /// callable, even when paused.
    pub fn claim_bounty(&mut self, id: u64) {
        let caller = self.env().caller();
        let bounty = self
            .bounties
            .get(&id)
            .unwrap_or_revert_with(&self.env(), StakingError::BountyNotFound);

        if !self.bounty_finalized.get_or_default(&id) {
            self.env().revert(StakingError::BountyNotFinalized);
        }
        if self.bounty_claimed.get_or_default(&(id, caller)) {
            self.env().revert(StakingError::AlreadyClaimed);
        }
        if !self.is_bounty_eligible(&bounty, &caller) {
            self.env().revert(StakingError::NotEligible);
        }

        let total = self.bounty_total_eligible_stake.get_or_default(&id);
        if total.is_zero() {
            self.env().revert(StakingError::NotEligible);
        }

        let snapshot = self
            .epoch_stake_snapshot
            .get_or_default(&(bounty.start_epoch, caller));
        let share = bounty.amount * snapshot / total;

        self.bounty_claimed.set(&(id, caller), true);
        if !share.is_zero() {
            self.token.transfer(&caller, &share);
        }

        self.env().emit_event(BountyClaimed {
            id,
            provider: caller,
            amount: share,
        });
    }

    // --- Read-only getters ---

    pub fn get_token(&self) -> Address {
        *self.token.address()
    }

    pub fn get_genesis_epoch(&self) -> u64 {
        self.genesis_epoch.get_or_default()
    }

    pub fn current_epoch(&self) -> u64 {
        self.env().get_block_time_secs() / EPOCH_LENGTH
    }

    pub fn get_min_stake(&self) -> U256 {
        self.min_stake.get_or_default()
    }

    pub fn get_uptime_threshold(&self) -> u32 {
        self.uptime_threshold.get_or_default()
    }

    pub fn get_reward_per_epoch(&self) -> U256 {
        self.reward_per_epoch.get_or_default()
    }

    pub fn get_reward_pool(&self) -> U256 {
        self.reward_pool.get_or_default()
    }

    pub fn get_active_stake(&self, provider: &Address) -> U256 {
        self.active_stake.get_or_default(provider)
    }

    pub fn get_pending_unstake(&self, provider: &Address) -> Option<PendingUnstake> {
        self.pending_unstake.get_or_default(provider)
    }

    pub fn get_epoch_reports(&self, epoch: u64, provider: &Address) -> u32 {
        self.epoch_reports.get_or_default(&(epoch, *provider))
    }

    pub fn get_epoch_threshold(&self, epoch: u64) -> Option<u32> {
        self.epoch_threshold.get(&epoch)
    }

    pub fn is_finalized(&self, epoch: u64) -> bool {
        self.epoch_finalized.get_or_default(&epoch)
    }

    pub fn get_epoch_total_eligible_stake(&self, epoch: u64) -> U256 {
        self.epoch_total_eligible_stake.get_or_default(&epoch)
    }

    pub fn get_bounty(&self, id: u64) -> Option<Bounty> {
        self.bounties.get(&id)
    }

    pub fn is_bounty_finalized(&self, id: u64) -> bool {
        self.bounty_finalized.get_or_default(&id)
    }
}

// --- Internal helpers ---

impl StakingRewards {
    fn transfer_to_contract(&mut self, amount: &U256) {
        let caller = self.env().caller();
        let self_address = self.env().self_address();
        self.token.transfer_from(&caller, &self_address, amount);
    }

    fn assert_admin(&self) {
        if !self.has_role(&StakingRole::Admin.role_id(), &self.env().caller()) {
            self.env().revert(StakingError::NotAdmin);
        }
    }

    fn assert_reporter(&self) {
        if !self.has_role(&StakingRole::Reporter.role_id(), &self.env().caller()) {
            self.env().revert(StakingError::NotReporter);
        }
    }

    /// A provider is bounty-eligible if it met the uptime gate for every epoch
    /// the window touches and its most recent report is at/after the window
    /// start. This approximates "reported at least once in `[start, end]`" with
    /// the per-epoch data we track (see module docs).
    fn is_bounty_eligible(&self, bounty: &Bounty, provider: &Address) -> bool {
        let mut epoch = bounty.start_epoch;
        loop {
            let threshold = self.epoch_threshold.get_or_default(&epoch);
            let reports = self.epoch_reports.get_or_default(&(epoch, *provider));
            if reports < threshold || reports == 0 {
                return false;
            }
            if epoch == bounty.end_epoch {
                break;
            }
            epoch += 1;
        }

        self.last_report_time.get_or_default(provider) >= bounty.start
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{StyksToken, StyksTokenHostRef};
    use odra::host::{Deployer, HostEnv, NoArgs};

    /// One epoch in milliseconds (the host clock advances in ms).
    const EPOCH_MS: u64 = EPOCH_LENGTH * 1000;

    struct Setup {
        env: HostEnv,
        token: StyksTokenHostRef,
        staking: StakingRewardsHostRef,
        admin: Address,
        provider_a: Address,
        provider_b: Address,
        supplier: Address,
    }

    fn setup() -> Setup {
        let env = odra_test::env();
        let admin = env.get_account(0);
        let provider_a = env.get_account(1);
        let provider_b = env.get_account(2);
        let supplier = env.get_account(3);

        // Deploy token (admin gets the whole initial supply).
        env.set_caller(admin);
        let token = StyksToken::deploy(&env, NoArgs);

        // Deploy staking, wired to the token.
        let staking = StakingRewards::deploy(
            &env,
            StakingRewardsInitArgs {
                token: token.address(),
            },
        );

        Setup {
            env,
            token,
            staking,
            admin,
            provider_a,
            provider_b,
            supplier,
        }
    }

    /// Transfer `whole` STYKS from admin to `to`, have `to` approve staking,
    /// then stake it as `to`.
    fn fund_and_stake(s: &mut Setup, to: Address, whole: u64) {
        let amount = styks(whole);
        s.env.set_caller(s.admin);
        s.token.transfer(&to, &amount);

        s.env.set_caller(to);
        s.token.approve(&s.staking.address(), &amount);
        s.staking.stake(amount);
    }

    fn fund_pool(s: &mut Setup, whole: u64) {
        let amount = styks(whole);
        s.env.set_caller(s.admin);
        s.token.approve(&s.staking.address(), &amount);
        s.staking.fund_pool(amount);
    }

    #[test]
    fn init_defaults() {
        let s = setup();
        assert_eq!(s.staking.get_token(), s.token.address());
        assert_eq!(s.staking.get_min_stake(), styks(10_000));
        assert_eq!(s.staking.get_uptime_threshold(), 24);
        assert_eq!(s.staking.get_reward_per_epoch(), styks(100));
        assert_eq!(s.staking.get_reward_pool(), U256::zero());
    }

    #[test]
    fn stake_registers_above_min() {
        let mut s = setup();
        let a = s.provider_a;

        // Below min_stake: held but not registered.
        fund_and_stake(&mut s, a, 5_000);
        assert_eq!(s.staking.get_active_stake(&s.provider_a), styks(5_000));

        // Top-up over the floor: now registered.
        fund_and_stake(&mut s, a, 5_000);
        assert_eq!(s.staking.get_active_stake(&s.provider_a), styks(10_000));
    }

    #[test]
    fn full_reward_cycle() {
        let mut s = setup();
        let (a, b) = (s.provider_a, s.provider_b);
        fund_pool(&mut s, 1_000);
        fund_and_stake(&mut s, a, 10_000);
        fund_and_stake(&mut s, b, 30_000);

        // Grant the supplier the Reporter role.
        s.env.set_caller(s.admin);
        s.staking
            .grant_role(&StakingRole::Reporter.role_id(), &s.supplier);

        // Both providers report in the genesis epoch (threshold 0).
        s.env.set_caller(s.supplier);
        s.staking.record_report(s.provider_a);
        s.staking.record_report(s.provider_b);
        assert_eq!(s.staking.get_epoch_reports(0, &s.provider_a), 1);

        // Move past the epoch and finalize.
        s.env.advance_block_time(EPOCH_MS);
        s.env.set_caller(s.provider_a);
        s.staking.finalize(0);
        assert!(s.staking.is_finalized(0));
        // Eligible stake = 10k + 30k = 40k.
        assert_eq!(s.staking.get_epoch_total_eligible_stake(0), styks(40_000));

        // Claim: budget 100 STYKS split 1:3 by stake.
        let before_a = s.token.balance_of(&s.provider_a);
        s.env.set_caller(s.provider_a);
        s.staking.claim(0);
        assert_eq!(
            s.token.balance_of(&s.provider_a) - before_a,
            styks(25) // 100 * 10k / 40k
        );

        let before_b = s.token.balance_of(&s.provider_b);
        s.env.set_caller(s.provider_b);
        s.staking.claim(0);
        assert_eq!(s.token.balance_of(&s.provider_b) - before_b, styks(75));

        // Pool drew exactly one epoch budget.
        assert_eq!(s.staking.get_reward_pool(), styks(900));

        // Double claim reverts.
        assert_eq!(
            s.staking.try_claim(0),
            Err(StakingError::AlreadyClaimed.into())
        );
    }

    #[test]
    fn unstake_floor_and_withdraw() {
        let mut s = setup();
        let a = s.provider_a;
        fund_and_stake(&mut s, a, 15_000);

        // Cannot leave active in (0, min_stake).
        s.env.set_caller(s.provider_a);
        assert_eq!(
            s.staking.try_request_unstake(styks(10_000)),
            Err(StakingError::UnstakeBelowFloor.into())
        );

        // Partial down to exactly min_stake is fine.
        s.staking.request_unstake(styks(5_000));
        assert_eq!(s.staking.get_active_stake(&s.provider_a), styks(10_000));
        assert!(s.staking.get_pending_unstake(&s.provider_a).is_some());

        // One pending slot only.
        assert_eq!(
            s.staking.try_request_unstake(styks(1_000)),
            Err(StakingError::PendingUnstakeExists.into())
        );

        // Locked until cooldown elapses.
        assert_eq!(
            s.staking.try_withdraw(),
            Err(StakingError::UnstakeStillLocked.into())
        );

        s.env.advance_block_time(7 * EPOCH_MS);
        let before = s.token.balance_of(&s.provider_a);
        s.staking.withdraw();
        assert_eq!(s.token.balance_of(&s.provider_a) - before, styks(5_000));
        assert!(s.staking.get_pending_unstake(&s.provider_a).is_none());
    }

    #[test]
    fn slash_caps_and_recycles() {
        let mut s = setup();
        let a = s.provider_a;
        fund_and_stake(&mut s, a, 20_000);

        // Cannot slash more than 50%.
        s.env.set_caller(s.admin);
        assert_eq!(
            s.staking.try_slash(s.provider_a, styks(10_001), [0u8; 32]),
            Err(StakingError::SlashExceedsCap.into())
        );

        // Slash 50% → recycled into the reward pool.
        s.staking.slash(s.provider_a, styks(10_000), [7u8; 32]);
        assert_eq!(s.staking.get_active_stake(&s.provider_a), styks(10_000));
        assert_eq!(s.staking.get_reward_pool(), styks(10_000));

        // At most one slash per epoch.
        assert_eq!(
            s.staking.try_slash(s.provider_a, styks(1_000), [7u8; 32]),
            Err(StakingError::AlreadySlashedThisEpoch.into())
        );
    }

    #[test]
    fn record_report_requires_reporter_role() {
        let mut s = setup();
        let a = s.provider_a;
        fund_and_stake(&mut s, a, 10_000);

        // Supplier without the role cannot credit reports.
        s.env.set_caller(s.supplier);
        assert_eq!(
            s.staking.try_record_report(s.provider_a),
            Err(StakingError::NotReporter.into())
        );
    }

    #[test]
    fn bounty_split_by_stake() {
        let mut s = setup();
        let (a, b) = (s.provider_a, s.provider_b);
        fund_and_stake(&mut s, a, 10_000);
        fund_and_stake(&mut s, b, 10_000);

        s.env.set_caller(s.admin);
        s.staking
            .grant_role(&StakingRole::Reporter.role_id(), &s.supplier);

        // A depositor funds a 1-hour bounty of 200 STYKS.
        let amount = styks(200);
        s.env.set_caller(s.admin);
        s.token.approve(&s.staking.address(), &amount);
        let bounty_id = s.staking.deposit_with_timedelta(amount, 3_600);

        // Only provider_a reports within the window.
        s.env.set_caller(s.supplier);
        s.staking.record_report(s.provider_a);

        // After the window, finalize and claim.
        s.env.advance_block_time(3_600 * 1000);
        s.env.set_caller(s.provider_a);
        s.staking.finalize_bounty(bounty_id);
        assert!(s.staking.is_bounty_finalized(bounty_id));

        let before = s.token.balance_of(&s.provider_a);
        s.staking.claim_bounty(bounty_id);
        // provider_a is the only eligible provider → takes the whole bounty.
        assert_eq!(s.token.balance_of(&s.provider_a) - before, styks(200));

        // provider_b never reported → not eligible.
        s.env.set_caller(s.provider_b);
        assert_eq!(
            s.staking.try_claim_bounty(bounty_id),
            Err(StakingError::NotEligible.into())
        );
    }
}
