#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod amm_pool {

    use ink_storage::{
        traits::SpreadAllocate,
        Mapping,
    };

    /// Constant product AMM (x * y = k)
    /// Provides a swap venue for the rebalancing vault

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct AmmPool {
        owner: AccountId,
        /// Reserve of token A (POT) in the pool
        reserve_a: Balance,
        /// Reserve of token B (rUSD) in the pool  
        reserve_b: Balance,
        /// Total LP shares issued
        total_lp: Balance,
        /// LP balances per provider
        lp_balances: Mapping<AccountId, Balance>,
        /// Fee in basis points (30 = 0.3%)
        fee_bps: u32,
        /// Name of token A
        token_a_name: [u8; 8],
        /// Name of token B
        token_b_name: [u8; 8],
        /// Cumulative volume for analytics
        volume_a: Balance,
        volume_b: Balance,
        /// Authorized rebalancer (the vault contract)
        rebalancer: Option<AccountId>,
    }

    #[ink(event)]
    pub struct LiquidityAdded {
        #[ink(topic)]
        provider: AccountId,
        amount_a: Balance,
        amount_b: Balance,
        lp_minted: Balance,
    }

    #[ink(event)]
    pub struct LiquidityRemoved {
        #[ink(topic)]
        provider: AccountId,
        amount_a: Balance,
        amount_b: Balance,
        lp_burned: Balance,
    }

    #[ink(event)]
    pub struct Swapped {
        #[ink(topic)]
        trader: AccountId,
        a_to_b: bool,
        amount_in: Balance,
        amount_out: Balance,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotOwner,
        InsufficientLiquidity,
        InsufficientInput,
        InsufficientOutput,
        SlippageExceeded,
        ZeroAmount,
        InsufficientLpBalance,
        TransferFailed,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl AmmPool {

        #[ink(constructor)]
        pub fn new(fee_bps: u32) -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.owner = Self::env().caller();
                contract.reserve_a = 0;
                contract.reserve_b = 0;
                contract.total_lp = 0;
                contract.fee_bps = fee_bps;
                contract.volume_a = 0;
                contract.volume_b = 0;
                contract.rebalancer = None;
                // POT  as token A, rUSD as token B
                contract.token_a_name = *b"POT     ";
                contract.token_b_name = *b"rUSD    ";
            })
        }

        /// Add liquidity — must send equal value of both tokens
        /// For simplicity: amount_a is sent as payable value,
        /// amount_b is tracked internally (simulated for hackathon)
        #[ink(message, payable)]
        pub fn add_liquidity(&mut self, amount_b: Balance) -> Result<Balance> {
            let amount_a = self.env().transferred_value();
            if amount_a == 0 || amount_b == 0 {
                return Err(Error::ZeroAmount);
            }

            let caller = self.env().caller();

            let lp_minted = if self.total_lp == 0 {
                // First liquidity — LP = sqrt(a * b) approximated as geometric mean
                self.integer_sqrt(amount_a.saturating_mul(amount_b))
            } else {
                // Proportional to existing pool
                let lp_from_a = amount_a
                    .saturating_mul(self.total_lp)
                    .checked_div(self.reserve_a)
                    .unwrap_or(0);
                let lp_from_b = amount_b
                    .saturating_mul(self.total_lp)
                    .checked_div(self.reserve_b)
                    .unwrap_or(0);
                lp_from_a.min(lp_from_b)
            };

            if lp_minted == 0 {
                return Err(Error::InsufficientLiquidity);
            }

            self.reserve_a = self.reserve_a.saturating_add(amount_a);
            self.reserve_b = self.reserve_b.saturating_add(amount_b);
            self.total_lp  = self.total_lp.saturating_add(lp_minted);

            let current_lp = self.lp_balances.get(caller).unwrap_or(0);
            self.lp_balances.insert(caller, &current_lp.saturating_add(lp_minted));

            self.env().emit_event(LiquidityAdded {
                provider: caller,
                amount_a,
                amount_b,
                lp_minted,
            });

            Ok(lp_minted)
        }

        /// Remove liquidity — burn LP shares, receive proportional tokens
        #[ink(message)]
        pub fn remove_liquidity(&mut self, lp_amount: Balance) -> Result<(Balance, Balance)> {
            let caller = self.env().caller();
            let current_lp = self.lp_balances.get(caller).unwrap_or(0);

            if lp_amount == 0 || lp_amount > current_lp {
                return Err(Error::InsufficientLpBalance);
            }
            if self.total_lp == 0 {
                return Err(Error::InsufficientLiquidity);
            }

            let amount_a = lp_amount
                .saturating_mul(self.reserve_a)
                .checked_div(self.total_lp)
                .unwrap_or(0);
            let amount_b = lp_amount
                .saturating_mul(self.reserve_b)
                .checked_div(self.total_lp)
                .unwrap_or(0);

            self.lp_balances.insert(caller, &current_lp.saturating_sub(lp_amount));
            self.total_lp  = self.total_lp.saturating_sub(lp_amount);
            self.reserve_a = self.reserve_a.saturating_sub(amount_a);
            self.reserve_b = self.reserve_b.saturating_sub(amount_b);

            // Transfer token A back (native POT)
            self.env().transfer(caller, amount_a)
                .map_err(|_| Error::TransferFailed)?;

            self.env().emit_event(LiquidityRemoved {
                provider: caller,
                amount_a,
                amount_b,
                lp_burned: lp_amount,
            });

            Ok((amount_a, amount_b))
        }

        /// Swap token A (POT) for token B (rUSD)
        #[ink(message, payable)]
        pub fn swap_a_for_b(&mut self, min_amount_out: Balance) -> Result<Balance> {
            let amount_in = self.env().transferred_value();
            if amount_in == 0 {
                return Err(Error::ZeroAmount);
            }
            if self.reserve_a == 0 || self.reserve_b == 0 {
                return Err(Error::InsufficientLiquidity);
            }

            let amount_out = self.get_amount_out(amount_in, self.reserve_a, self.reserve_b)?;

            if amount_out < min_amount_out {
                return Err(Error::SlippageExceeded);
            }

            self.reserve_a = self.reserve_a.saturating_add(amount_in);
            self.reserve_b = self.reserve_b.saturating_sub(amount_out);
            self.volume_a  = self.volume_a.saturating_add(amount_in);

            self.env().emit_event(Swapped {
                trader: self.env().caller(),
                a_to_b: true,
                amount_in,
                amount_out,
            });

            Ok(amount_out)
        }

        /// Swap token B (rUSD) for token A (POT) — simulated
        #[ink(message)]
        pub fn swap_b_for_a(&mut self, amount_in: Balance, min_amount_out: Balance) -> Result<Balance> {
            if amount_in == 0 {
                return Err(Error::ZeroAmount);
            }
            if self.reserve_a == 0 || self.reserve_b == 0 {
                return Err(Error::InsufficientLiquidity);
            }

            let amount_out = self.get_amount_out(amount_in, self.reserve_b, self.reserve_a)?;

            if amount_out < min_amount_out {
                return Err(Error::SlippageExceeded);
            }

            self.reserve_b = self.reserve_b.saturating_add(amount_in);
            self.reserve_a = self.reserve_a.saturating_sub(amount_out);
            self.volume_b  = self.volume_b.saturating_add(amount_in);

            // Transfer POT out
            self.env().transfer(self.env().caller(), amount_out)
                .map_err(|_| Error::TransferFailed)?;

            self.env().emit_event(Swapped {
                trader: self.env().caller(),
                a_to_b: false,
                amount_in,
                amount_out,
            });

            Ok(amount_out)
        }

        /// Quote: how much token B do I get for amount_in of token A?
        #[ink(message)]
        pub fn quote_a_for_b(&self, amount_in: Balance) -> Result<Balance> {
            self.get_amount_out(amount_in, self.reserve_a, self.reserve_b)
        }

        /// Quote: how much token A do I get for amount_in of token B?
        #[ink(message)]
        pub fn quote_b_for_a(&self, amount_in: Balance) -> Result<Balance> {
            self.get_amount_out(amount_in, self.reserve_b, self.reserve_a)
        }

        /// Get current spot price of A in terms of B (× 1_000_000)
        #[ink(message)]
        pub fn get_spot_price(&self) -> u128 {
            if self.reserve_a == 0 {
                return 0;
            }
            self.reserve_b
                .saturating_mul(1_000_000)
                .checked_div(self.reserve_a)
                .unwrap_or(0)
        }

        /// Get pool reserves
        #[ink(message)]
        pub fn get_reserves(&self) -> (Balance, Balance) {
            (self.reserve_a, self.reserve_b)
        }

        /// Get LP balance for an account
        #[ink(message)]
        pub fn get_lp_balance(&self, account: AccountId) -> Balance {
            self.lp_balances.get(account).unwrap_or(0)
        }

        /// Get pool analytics
        #[ink(message)]
        pub fn get_stats(&self) -> (Balance, Balance, Balance, Balance, Balance) {
            (self.reserve_a, self.reserve_b, self.total_lp, self.volume_a, self.volume_b)
        }

        /// Set authorized rebalancer (the vault contract address)
        #[ink(message)]
        pub fn set_rebalancer(&mut self, rebalancer: AccountId) -> Result<()> {
            self.ensure_owner()?;
            self.rebalancer = Some(rebalancer);
            Ok(())
        }

        // ─── INTERNAL ──────────────────────────────────────────────────────────

        /// Constant product formula with fee
        /// amount_out = (amount_in × (10000 - fee) × reserve_out)
        ///              / (reserve_in × 10000 + amount_in × (10000 - fee))
        fn get_amount_out(
            &self,
            amount_in: Balance,
            reserve_in: Balance,
            reserve_out: Balance,
        ) -> Result<Balance> {
            if reserve_in == 0 || reserve_out == 0 {
                return Err(Error::InsufficientLiquidity);
            }
            let fee_factor = (10_000u128 - self.fee_bps as u128) as Balance;
            let amount_in_with_fee = amount_in.saturating_mul(fee_factor);
            let numerator = amount_in_with_fee.saturating_mul(reserve_out);
            let denominator = reserve_in
                .saturating_mul(10_000)
                .saturating_add(amount_in_with_fee);
            Ok(numerator.checked_div(denominator).unwrap_or(0))
        }

        /// Integer square root (Babylonian method)
        fn integer_sqrt(&self, n: u128) -> u128 {
            if n == 0 { return 0; }
            let mut x = n;
            let mut y = x.saturating_add(1) / 2;
            while y < x {
                x = y;
                y = (x.saturating_add(n / x)) / 2;
            }
            x
        }

        fn ensure_owner(&self) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn quote_works() {
            let pool = AmmPool::new(30); // 0.3% fee
            // With reserves 1000:1000, swapping 100 A
            // should give slightly less than 100 B due to slippage + fee
            // (tested after add_liquidity in integration)
            assert_eq!(pool.get_spot_price(), 0); // empty pool
        }
    }
}