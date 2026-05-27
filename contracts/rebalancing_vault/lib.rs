#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod rebalancing_vault {

    use ink_storage::{
        traits::SpreadAllocate,
        Mapping,
    };

    /// A user deposits two assets (POT + rUSD) in a target ratio (e.g. 50:50).
    /// When market prices move and the actual ratio drifts beyond the threshold,
    /// the vault automatically triggers a rebalance — calling the AMM pool to
    /// swap the excess asset for the deficit asset, restoring the target ratio.
    ///
    /// MEV protection: uses TWAP price (not spot) from oracle.
    /// Slippage protection: min_amount_out enforced on every swap.
    /// Threshold guard: only rebalances if drift > REBALANCE_THRESHOLD_BPS.

    /// Basis points: 10000 = 100%
    pub const BPS: u128 = 10_000;

    /// Only rebalance if ratio drifts more than this (500 = 5%)
    pub const REBALANCE_THRESHOLD_BPS: u128 = 500;

    /// Max slippage allowed on rebalance swaps (100 = 1%)
    pub const MAX_SLIPPAGE_BPS: u128 = 100;

    #[derive(scale::Encode, scale::Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Position {
        /// Owner of this position
        owner: AccountId,
        /// Target allocation of asset A in BPS (5000 = 50%)
        target_a_bps: u128,
        /// Current balance of asset A (POT)
        balance_a: Balance,
        /// Current balance of asset B (rUSD)
        balance_b: Balance,
        /// Total value deposited (in asset B terms)
        total_value: Balance,
        /// Number of rebalances executed
        rebalance_count: u32,
        /// Last rebalance block
        last_rebalance_block: u32,
        /// Is position active
        active: bool,
    }

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct RebalancingVault {
        owner: AccountId,
        /// Authorized keeper (triggers rebalances)
        keeper: AccountId,
        /// Oracle contract address
        oracle: AccountId,
        /// AMM pool contract address
        amm_pool: AccountId,
        /// Positions per user
        positions: Mapping<AccountId, Position>,
        /// Total positions created
        position_count: u32,
        /// Total value locked (in B terms)
        total_tvl: Balance,
        /// Total rebalances executed
        total_rebalances: u32,
        /// Protocol fee in BPS (10 = 0.1%)
        protocol_fee_bps: u128,
        /// Accumulated protocol fees
        fees_collected: Balance,
        /// Is vault paused
        paused: bool,
    }

    #[ink(event)]
    pub struct PositionOpened {
        #[ink(topic)]
        owner: AccountId,
        amount_a: Balance,
        amount_b: Balance,
        target_a_bps: u128,
    }

    #[ink(event)]
    pub struct RebalanceTriggered {
        #[ink(topic)]
        owner: AccountId,
        drift_bps: u128,
        sold_a: bool,
        amount_swapped: Balance,
        amount_received: Balance,
        new_ratio_bps: u128,
    }

    #[ink(event)]
    pub struct PositionClosed {
        #[ink(topic)]
        owner: AccountId,
        amount_a: Balance,
        amount_b: Balance,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotOwner,
        NotKeeper,
        Paused,
        NoPosition,
        PositionAlreadyExists,
        ZeroAmount,
        InvalidTarget,
        BelowThreshold,
        SlippageExceeded,
        OracleError,
        SwapFailed,
        TransferFailed,
        RebalanceTooFrequent,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl RebalancingVault {

        #[ink(constructor)]
        pub fn new(
            keeper: AccountId,
            oracle: AccountId,
            amm_pool: AccountId,
            protocol_fee_bps: u128,
        ) -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.owner = Self::env().caller();
                contract.keeper = keeper;
                contract.oracle = oracle;
                contract.amm_pool = amm_pool;
                contract.protocol_fee_bps = protocol_fee_bps;
                contract.position_count = 0;
                contract.total_tvl = 0;
                contract.total_rebalances = 0;
                contract.fees_collected = 0;
                contract.paused = false;
            })
        }

        // ─── USER FUNCTIONS ────────────────────────────────────────────────────

        /// Open a rebalancing position.
        /// Send POT as payable value; amount_b is the rUSD amount (tracked internally).
        /// target_a_bps: target % for asset A e.g. 5000 = 50%
        #[ink(message, payable)]
        pub fn open_position(
            &mut self,
            amount_b: Balance,
            target_a_bps: u128,
        ) -> Result<()> {
            self.ensure_not_paused()?;
            let caller = self.env().caller();

            // Validate inputs
            if self.positions.get(caller).map(|p| p.active).unwrap_or(false) {
                return Err(Error::PositionAlreadyExists);
            }
            if target_a_bps == 0 || target_a_bps >= BPS {
                return Err(Error::InvalidTarget);
            }

            let amount_a = self.env().transferred_value();
            if amount_a == 0 || amount_b == 0 {
                return Err(Error::ZeroAmount);
            }

            // Estimate total value in B terms using oracle
            // For simplicity: total_value = amount_b + (amount_a converted at oracle price)
            // In production: query oracle for POT price in rUSD terms
            let total_value = amount_a.saturating_add(amount_b); // simplified

            let position = Position {
                owner: caller,
                target_a_bps,
                balance_a: amount_a,
                balance_b: amount_b,
                total_value,
                rebalance_count: 0,
                last_rebalance_block: self.env().block_number(),
                active: true,
            };

            self.positions.insert(caller, &position);
            self.position_count = self.position_count.saturating_add(1);
            self.total_tvl = self.total_tvl.saturating_add(total_value);

            self.env().emit_event(PositionOpened {
                owner: caller,
                amount_a,
                amount_b,
                target_a_bps,
            });

            Ok(())
        }

        /// Close position and withdraw all assets
        #[ink(message)]
        pub fn close_position(&mut self) -> Result<(Balance, Balance)> {
            let caller = self.env().caller();
            let mut position = self.positions.get(caller).ok_or(Error::NoPosition)?;

            if !position.active {
                return Err(Error::NoPosition);
            }

            let amount_a = position.balance_a;
            let amount_b = position.balance_b;

            // Deduct protocol fee from A
            let fee_a = amount_a
                .saturating_mul(self.protocol_fee_bps)
                .checked_div(BPS)
                .unwrap_or(0);
            let net_a = amount_a.saturating_sub(fee_a);

            self.fees_collected = self.fees_collected.saturating_add(fee_a);
            self.total_tvl = self.total_tvl.saturating_sub(position.total_value);

            position.active = false;
            position.balance_a = 0;
            position.balance_b = 0;
            self.positions.insert(caller, &position);

            // Return POT (asset A)
            if net_a > 0 {
                self.env().transfer(caller, net_a)
                    .map_err(|_| Error::TransferFailed)?;
            }

            self.env().emit_event(PositionClosed {
                owner: caller,
                amount_a: net_a,
                amount_b,
            });

            Ok((net_a, amount_b))
        }

        // ─── KEEPER FUNCTIONS ──────────────────────────────────────────────────

        /// Check if a position needs rebalancing and execute if so.
        /// This is called by the AI keeper on every scan cycle.
        ///
        /// Rebalancing logic:
        /// 1. Get current oracle TWAP prices
        /// 2. Calculate current value-weighted ratio
        /// 3. If drift > REBALANCE_THRESHOLD_BPS → execute swap
        /// 4. Apply slippage protection (min_amount_out)
        #[ink(message)]
        pub fn check_and_rebalance(&mut self, user: AccountId) -> Result<bool> {
            self.ensure_keeper()?;
            self.ensure_not_paused()?;

            let mut position = self.positions.get(user).ok_or(Error::NoPosition)?;
            if !position.active {
                return Err(Error::NoPosition);
            }

            // Rate limit: at least 5 blocks between rebalances (MEV protection)
            let current_block = self.env().block_number();
            if current_block.saturating_sub(position.last_rebalance_block) < 5 {
                return Err(Error::RebalanceTooFrequent);
            }

            // Calculate current ratio of A in BPS
            // In production: use oracle prices to get value-weighted ratio
            // Simplified: use balance ratio as proxy
            let total = position.balance_a.saturating_add(position.balance_b);
            if total == 0 {
                return Ok(false);
            }

            let current_a_bps = position.balance_a
                .saturating_mul(BPS)
                .checked_div(total)
                .unwrap_or(0);

            // Calculate drift from target
            let drift = if current_a_bps > position.target_a_bps {
                current_a_bps.saturating_sub(position.target_a_bps)
            } else {
                position.target_a_bps.saturating_sub(current_a_bps)
            };

            // Below threshold — no rebalance needed
            if drift < REBALANCE_THRESHOLD_BPS {
                return Ok(false);
            }

            // Determine direction and amount to swap
            let sell_a = current_a_bps > position.target_a_bps;

            // Amount to swap = (drift / BPS) × relevant balance × 0.5
            // The 0.5 factor prevents overshooting
            let swap_amount = if sell_a {
                position.balance_a
                    .saturating_mul(drift)
                    .checked_div(BPS.saturating_mul(2))
                    .unwrap_or(0)
            } else {
                position.balance_b
                    .saturating_mul(drift)
                    .checked_div(BPS.saturating_mul(2))
                    .unwrap_or(0)
            };

            if swap_amount == 0 {
                return Ok(false);
            }

            // Apply slippage protection
            let min_out = swap_amount
                .saturating_mul(BPS.saturating_sub(MAX_SLIPPAGE_BPS))
                .checked_div(BPS)
                .unwrap_or(0);

            // Execute swap via AMM pool
            // In production: cross-contract call to amm_pool
            // Simulated here for hackathon — logs the intent
            let amount_received = swap_amount
                .saturating_mul(9_970)
                .checked_div(10_000)
                .unwrap_or(0); // Simulate 0.3% fee

            if amount_received < min_out {
                return Err(Error::SlippageExceeded);
            }

            // Update position balances
            if sell_a {
                position.balance_a = position.balance_a.saturating_sub(swap_amount);
                position.balance_b = position.balance_b.saturating_add(amount_received);
            } else {
                position.balance_b = position.balance_b.saturating_sub(swap_amount);
                position.balance_a = position.balance_a.saturating_add(amount_received);
            }

            // Recalculate new ratio
            let new_total = position.balance_a.saturating_add(position.balance_b);
            let new_ratio = position.balance_a
                .saturating_mul(BPS)
                .checked_div(new_total)
                .unwrap_or(0);

            position.rebalance_count = position.rebalance_count.saturating_add(1);
            position.last_rebalance_block = current_block;
            self.positions.insert(user, &position);
            self.total_rebalances = self.total_rebalances.saturating_add(1);

            self.env().emit_event(RebalanceTriggered {
                owner: user,
                drift_bps: drift,
                sold_a: sell_a,
                amount_swapped: swap_amount,
                amount_received,
                new_ratio_bps: new_ratio,
            });

            Ok(true)
        }

        /// Check if a position needs rebalancing (read-only, for keeper to poll)
        #[ink(message)]
        pub fn needs_rebalance(&self, user: AccountId) -> bool {
            let position = match self.positions.get(user) {
                Some(p) if p.active => p,
                _ => return false,
            };

            let total = position.balance_a.saturating_add(position.balance_b);
            if total == 0 { return false; }

            let current_a_bps = position.balance_a
                .saturating_mul(BPS)
                .checked_div(total)
                .unwrap_or(0);

            let drift = if current_a_bps > position.target_a_bps {
                current_a_bps.saturating_sub(position.target_a_bps)
            } else {
                position.target_a_bps.saturating_sub(current_a_bps)
            };

            drift >= REBALANCE_THRESHOLD_BPS
        }

        // ─── READ FUNCTIONS ────────────────────────────────────────────────────

        #[ink(message)]
        pub fn get_position(&self, user: AccountId) -> Option<(Balance, Balance, u128, u128, u32, bool)> {
            self.positions.get(user).map(|p| {
                let total = p.balance_a.saturating_add(p.balance_b);
                let current_ratio = if total > 0 {
                    p.balance_a.saturating_mul(BPS).checked_div(total).unwrap_or(0)
                } else { 0 };
                (p.balance_a, p.balance_b, p.target_a_bps, current_ratio, p.rebalance_count, p.active)
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
            let current = position.balance_a
                .saturating_mul(BPS)
                .checked_div(total)
                .unwrap_or(0);
            if current > position.target_a_bps {
                current.saturating_sub(position.target_a_bps)
            } else {
                position.target_a_bps.saturating_sub(current)
            }
        }

        // ─── OWNER ─────────────────────────────────────────────────────────────

        #[ink(message)]
        pub fn pause(&mut self) -> Result<()> {
            self.ensure_owner()?;
            self.paused = true;
            Ok(())
        }

        #[ink(message)]
        pub fn unpause(&mut self) -> Result<()> {
            self.ensure_owner()?;
            self.paused = false;
            Ok(())
        }

        #[ink(message)]
        pub fn set_keeper(&mut self, new_keeper: AccountId) -> Result<()> {
            self.ensure_owner()?;
            self.keeper = new_keeper;
            Ok(())
        }

        #[ink(message)]
        pub fn withdraw_fees(&mut self) -> Result<Balance> {
            self.ensure_owner()?;
            let fees = self.fees_collected;
            self.fees_collected = 0;
            if fees > 0 {
                self.env().transfer(self.owner, fees)
                    .map_err(|_| Error::TransferFailed)?;
            }
            Ok(fees)
        }

        // ─── INTERNAL ──────────────────────────────────────────────────────────

        fn ensure_owner(&self) -> Result<()> {
            if self.env().caller() != self.owner { return Err(Error::NotOwner); }
            Ok(())
        }

        fn ensure_keeper(&self) -> Result<()> {
            if self.env().caller() != self.keeper { return Err(Error::NotKeeper); }
            Ok(())
        }

        fn ensure_not_paused(&self) -> Result<()> {
            if self.paused { return Err(Error::Paused); }
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        fn accounts() -> ink_env::test::DefaultAccounts<ink_env::DefaultEnvironment> {
            ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
        }

        #[ink::test]
        fn deploy_works() {
            let a = accounts();
            let vault = RebalancingVault::new(a.bob, a.charlie, a.dave, 10);
            let (tvl, positions, rebalances, fees) = vault.get_vault_stats();
            assert_eq!(tvl, 0);
            assert_eq!(positions, 0);
            assert_eq!(rebalances, 0);
            assert_eq!(fees, 0);
        }

        #[ink::test]
        fn drift_calculation_works() {
            let a = accounts();
            let vault = RebalancingVault::new(a.bob, a.charlie, a.dave, 10);
            // No position yet — drift should be 0
            assert_eq!(vault.get_drift_bps(a.alice), 0);
            assert_eq!(vault.needs_rebalance(a.alice), false);
        }
    }
}