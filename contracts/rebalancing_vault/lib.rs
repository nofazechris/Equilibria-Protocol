#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod rebalancing_vault {

    use ink::storage::Mapping;

    pub const BPS: u128 = 10_000;
    pub const REBALANCE_THRESHOLD_BPS: u128 = 500;
    pub const MAX_SLIPPAGE_BPS: u128 = 100;

    #[derive(scale::Encode, scale::Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Position {
        target_a_bps: u128,
        balance_a: Balance,
        balance_b: Balance,
        total_value: Balance,
        rebalance_count: u32,
        last_rebalance_block: u32,
        active: bool,
    }

    #[ink(storage)]
    pub struct RebalancingVault {
        owner: AccountId,
        keeper: AccountId,
        positions: Mapping<AccountId, Position>,
        position_count: u32,
        total_tvl: Balance,
        total_rebalances: u32,
        protocol_fee_bps: u128,
        fees_collected: Balance,
        paused: bool,
    }

    #[ink(event)]
    pub struct PositionOpened { #[ink(topic)] owner: AccountId, amount_a: Balance, amount_b: Balance, target_a_bps: u128 }
    #[ink(event)]
    pub struct RebalanceTriggered { #[ink(topic)] owner: AccountId, drift_bps: u128, sold_a: bool, amount_swapped: Balance, amount_received: Balance, new_ratio_bps: u128 }
    #[ink(event)]
    pub struct PositionClosed { #[ink(topic)] owner: AccountId, amount_a: Balance, amount_b: Balance }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotOwner, NotKeeper, Paused, NoPosition,
        PositionAlreadyExists, ZeroAmount, InvalidTarget,
        BelowThreshold, SlippageExceeded, TransferFailed, RebalanceTooFrequent,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl RebalancingVault {

        #[ink(constructor)]
        pub fn new(keeper: AccountId, protocol_fee_bps: u128) -> Self {
            Self {
                owner: Self::env().caller(),
                keeper,
                positions: Mapping::default(),
                position_count: 0,
                total_tvl: 0,
                total_rebalances: 0,
                protocol_fee_bps,
                fees_collected: 0,
                paused: false,
            }
        }

        #[ink(message, payable)]
        pub fn open_position(&mut self, amount_b: Balance, target_a_bps: u128) -> Result<()> {
            self.ensure_not_paused()?;
            let caller = self.env().caller();
            if self.positions.get(caller).map(|p| p.active).unwrap_or(false) {
                return Err(Error::PositionAlreadyExists);
            }
            if target_a_bps == 0 || target_a_bps >= BPS { return Err(Error::InvalidTarget); }
            let amount_a = self.env().transferred_value();
            if amount_a == 0 || amount_b == 0 { return Err(Error::ZeroAmount); }
            let total_value = amount_a.saturating_add(amount_b);
            let position = Position {
                target_a_bps, balance_a: amount_a, balance_b: amount_b,
                total_value, rebalance_count: 0,
                last_rebalance_block: self.env().block_number(),
                active: true,
            };
            self.positions.insert(caller, &position);
            self.position_count = self.position_count.saturating_add(1);
            self.total_tvl = self.total_tvl.saturating_add(total_value);
            self.env().emit_event(PositionOpened { owner: caller, amount_a, amount_b, target_a_bps });
            Ok(())
        }

        #[ink(message)]
        pub fn close_position(&mut self) -> Result<(Balance, Balance)> {
            let caller = self.env().caller();
            let mut position = self.positions.get(caller).ok_or(Error::NoPosition)?;
            if !position.active { return Err(Error::NoPosition); }
            let amount_a = position.balance_a;
            let amount_b = position.balance_b;
            let fee_a = amount_a.saturating_mul(self.protocol_fee_bps).checked_div(BPS).unwrap_or(0);
            let net_a = amount_a.saturating_sub(fee_a);
            self.fees_collected = self.fees_collected.saturating_add(fee_a);
            self.total_tvl = self.total_tvl.saturating_sub(position.total_value);
            position.active = false;
            position.balance_a = 0;
            position.balance_b = 0;
            self.positions.insert(caller, &position);
            if net_a > 0 {
                self.env().transfer(caller, net_a).map_err(|_| Error::TransferFailed)?;
            }
            self.env().emit_event(PositionClosed { owner: caller, amount_a: net_a, amount_b });
            Ok((net_a, amount_b))
        }

        #[ink(message)]
        pub fn check_and_rebalance(&mut self, user: AccountId) -> Result<bool> {
            self.ensure_keeper()?;
            self.ensure_not_paused()?;
            let mut position = self.positions.get(user).ok_or(Error::NoPosition)?;
            if !position.active { return Err(Error::NoPosition); }
            let current_block = self.env().block_number();
            if current_block.saturating_sub(position.last_rebalance_block) < 5 {
                return Err(Error::RebalanceTooFrequent);
            }
            let total = position.balance_a.saturating_add(position.balance_b);
            if total == 0 { return Ok(false); }
            let current_a_bps = position.balance_a.saturating_mul(BPS).checked_div(total).unwrap_or(0);
            let drift = if current_a_bps > position.target_a_bps {
                current_a_bps.saturating_sub(position.target_a_bps)
            } else {
                position.target_a_bps.saturating_sub(current_a_bps)
            };
            if drift < REBALANCE_THRESHOLD_BPS { return Ok(false); }
            let sell_a: bool = current_a_bps > position.target_a_bps;
            let sold_a = sell_a;
            let swap_amount = if sell_a {
                position.balance_a.saturating_mul(drift).checked_div(BPS.saturating_mul(2)).unwrap_or(0)
            } else {
                position.balance_b.saturating_mul(drift).checked_div(BPS.saturating_mul(2)).unwrap_or(0)
            };
            if swap_amount == 0 { return Ok(false); }
            let min_out = swap_amount.saturating_mul(BPS.saturating_sub(MAX_SLIPPAGE_BPS)).checked_div(BPS).unwrap_or(0);
            let amount_received = swap_amount.saturating_mul(9_970).checked_div(10_000).unwrap_or(0);
            if amount_received < min_out { return Err(Error::SlippageExceeded); }
            if sell_a {
                position.balance_a = position.balance_a.saturating_sub(swap_amount);
                position.balance_b = position.balance_b.saturating_add(amount_received);
            } else {
                position.balance_b = position.balance_b.saturating_sub(swap_amount);
                position.balance_a = position.balance_a.saturating_add(amount_received);
            }
            let new_total = position.balance_a.saturating_add(position.balance_b);
            let new_ratio = position.balance_a.saturating_mul(BPS).checked_div(new_total).unwrap_or(0);
            position.rebalance_count = position.rebalance_count.saturating_add(1);
            position.last_rebalance_block = current_block;
            self.positions.insert(user, &position);
            self.total_rebalances = self.total_rebalances.saturating_add(1);
            self.env().emit_event(RebalanceTriggered {
                owner: user, drift_bps: drift, sold_a,
                amount_swapped: swap_amount, amount_received, new_ratio_bps: new_ratio,
            });
            Ok(true)
        }

        #[ink(message)]
        pub fn needs_rebalance(&self, user: AccountId) -> bool {
            let position = match self.positions.get(user) {
                Some(p) if p.active => p,
                _ => return false,
            };
            let total = position.balance_a.saturating_add(position.balance_b);
            if total == 0 { return false; }
            let current_a_bps = position.balance_a.saturating_mul(BPS).checked_div(total).unwrap_or(0);
            let drift = if current_a_bps > position.target_a_bps {
                current_a_bps.saturating_sub(position.target_a_bps)
            } else {
                position.target_a_bps.saturating_sub(current_a_bps)
            };
            drift >= REBALANCE_THRESHOLD_BPS
        }

        #[ink(message)]
        pub fn get_position(&self, user: AccountId) -> Option<(Balance, Balance, u128, u128, u32, bool)> {
            self.positions.get(user).map(|p| {
                let total = p.balance_a.saturating_add(p.balance_b);
                let ratio = if total > 0 { p.balance_a.saturating_mul(BPS).checked_div(total).unwrap_or(0) } else { 0 };
                (p.balance_a, p.balance_b, p.target_a_bps, ratio, p.rebalance_count, p.active)
            })
        }

        #[ink(message)]
        pub fn get_vault_stats(&self) -> (Balance, u32, u32, Balance) {
            (self.total_tvl, self.position_count, self.total_rebalances, self.fees_collected)
        }

        #[ink(message)]
        pub fn get_drift_bps(&self, user: AccountId) -> u128 {
            let position = match self.positions.get(user) {
                Some(p) if p.active => p,
                _ => return 0,
            };
            let total = position.balance_a.saturating_add(position.balance_b);
            if total == 0 { return 0; }
            let current = position.balance_a.saturating_mul(BPS).checked_div(total).unwrap_or(0);
            if current > position.target_a_bps { current.saturating_sub(position.target_a_bps) }
            else { position.target_a_bps.saturating_sub(current) }
        }

        #[ink(message)] pub fn pause(&mut self) -> Result<()> { self.ensure_owner()?; self.paused = true; Ok(()) }
        #[ink(message)] pub fn unpause(&mut self) -> Result<()> { self.ensure_owner()?; self.paused = false; Ok(()) }
        #[ink(message)] pub fn set_keeper(&mut self, k: AccountId) -> Result<()> { self.ensure_owner()?; self.keeper = k; Ok(()) }
        #[ink(message)] pub fn get_owner(&self) -> AccountId { self.owner }
        #[ink(message)] pub fn get_keeper(&self) -> AccountId { self.keeper }
        #[ink(message)] pub fn is_paused(&self) -> bool { self.paused }

        fn ensure_owner(&self) -> Result<()> { if self.env().caller() != self.owner { return Err(Error::NotOwner); } Ok(()) }
        fn ensure_keeper(&self) -> Result<()> { if self.env().caller() != self.keeper { return Err(Error::NotKeeper); } Ok(()) }
        fn ensure_not_paused(&self) -> Result<()> { if self.paused { return Err(Error::Paused); } Ok(()) }
    }
}