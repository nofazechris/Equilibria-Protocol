#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod amm_pool {

    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct AmmPool {
        owner: AccountId,
        reserve_a: Balance,
        reserve_b: Balance,
        total_lp: Balance,
        lp_balances: Mapping<AccountId, Balance>,
        fee_bps: u32,
        volume_a: Balance,
        volume_b: Balance,
    }

    #[ink(event)]
    pub struct LiquidityAdded { provider: AccountId, amount_a: Balance, amount_b: Balance, lp_minted: Balance }
    #[ink(event)]
    pub struct LiquidityRemoved { provider: AccountId, amount_a: Balance, amount_b: Balance, lp_burned: Balance }
    #[ink(event)]
    pub struct Swapped { trader: AccountId, a_to_b: bool, amount_in: Balance, amount_out: Balance }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotOwner, InsufficientLiquidity, SlippageExceeded,
        ZeroAmount, InsufficientLpBalance, TransferFailed,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl AmmPool {

        #[ink(constructor)]
        pub fn new(fee_bps: u32) -> Self {
            Self {
                owner: Self::env().caller(),
                reserve_a: 0, reserve_b: 0, total_lp: 0,
                lp_balances: Mapping::default(),
                fee_bps, volume_a: 0, volume_b: 0,
            }
        }

        #[ink(message, payable)]
        pub fn add_liquidity(&mut self, amount_b: Balance) -> Result<Balance> {
            let amount_a = self.env().transferred_value();
            if amount_a == 0 || amount_b == 0 { return Err(Error::ZeroAmount); }
            let caller = self.env().caller();
            let lp_minted = if self.total_lp == 0 {
                self.integer_sqrt(amount_a.saturating_mul(amount_b))
            } else {
                let from_a = amount_a.saturating_mul(self.total_lp).checked_div(self.reserve_a).unwrap_or(0);
                let from_b = amount_b.saturating_mul(self.total_lp).checked_div(self.reserve_b).unwrap_or(0);
                from_a.min(from_b)
            };
            if lp_minted == 0 { return Err(Error::InsufficientLiquidity); }
            self.reserve_a = self.reserve_a.saturating_add(amount_a);
            self.reserve_b = self.reserve_b.saturating_add(amount_b);
            self.total_lp = self.total_lp.saturating_add(lp_minted);
            let cur = self.lp_balances.get(caller).unwrap_or(0);
            self.lp_balances.insert(caller, &cur.saturating_add(lp_minted));
            self.env().emit_event(LiquidityAdded { provider: caller, amount_a, amount_b, lp_minted });
            Ok(lp_minted)
        }

        #[ink(message)]
        pub fn remove_liquidity(&mut self, lp_amount: Balance) -> Result<(Balance, Balance)> {
            let caller = self.env().caller();
            let cur = self.lp_balances.get(caller).unwrap_or(0);
            if lp_amount == 0 || lp_amount > cur { return Err(Error::InsufficientLpBalance); }
            if self.total_lp == 0 { return Err(Error::InsufficientLiquidity); }
            let amount_a = lp_amount.saturating_mul(self.reserve_a).checked_div(self.total_lp).unwrap_or(0);
            let amount_b = lp_amount.saturating_mul(self.reserve_b).checked_div(self.total_lp).unwrap_or(0);
            self.lp_balances.insert(caller, &cur.saturating_sub(lp_amount));
            self.total_lp = self.total_lp.saturating_sub(lp_amount);
            self.reserve_a = self.reserve_a.saturating_sub(amount_a);
            self.reserve_b = self.reserve_b.saturating_sub(amount_b);
            self.env().transfer(caller, amount_a).map_err(|_| Error::TransferFailed)?;
            self.env().emit_event(LiquidityRemoved { provider: caller, amount_a, amount_b, lp_burned: lp_amount });
            Ok((amount_a, amount_b))
        }

        #[ink(message, payable)]
        pub fn swap_a_for_b(&mut self, min_out: Balance) -> Result<Balance> {
            let amount_in = self.env().transferred_value();
            if amount_in == 0 { return Err(Error::ZeroAmount); }
            if self.reserve_a == 0 || self.reserve_b == 0 { return Err(Error::InsufficientLiquidity); }
            let amount_out = self.get_amount_out(amount_in, self.reserve_a, self.reserve_b)?;
            if amount_out < min_out { return Err(Error::SlippageExceeded); }
            self.reserve_a = self.reserve_a.saturating_add(amount_in);
            self.reserve_b = self.reserve_b.saturating_sub(amount_out);
            self.volume_a = self.volume_a.saturating_add(amount_in);
            self.env().emit_event(Swapped { trader: self.env().caller(), a_to_b: true, amount_in, amount_out });
            Ok(amount_out)
        }

        #[ink(message)]
        pub fn swap_b_for_a(&mut self, amount_in: Balance, min_out: Balance) -> Result<Balance> {
            if amount_in == 0 { return Err(Error::ZeroAmount); }
            if self.reserve_a == 0 || self.reserve_b == 0 { return Err(Error::InsufficientLiquidity); }
            let amount_out = self.get_amount_out(amount_in, self.reserve_b, self.reserve_a)?;
            if amount_out < min_out { return Err(Error::SlippageExceeded); }
            self.reserve_b = self.reserve_b.saturating_add(amount_in);
            self.reserve_a = self.reserve_a.saturating_sub(amount_out);
            self.volume_b = self.volume_b.saturating_add(amount_in);
            self.env().transfer(self.env().caller(), amount_out).map_err(|_| Error::TransferFailed)?;
            self.env().emit_event(Swapped { trader: self.env().caller(), a_to_b: false, amount_in, amount_out });
            Ok(amount_out)
        }

        #[ink(message)]
        pub fn quote_a_for_b(&self, amount_in: Balance) -> Result<Balance> {
            self.get_amount_out(amount_in, self.reserve_a, self.reserve_b)
        }

        #[ink(message)]
        pub fn quote_b_for_a(&self, amount_in: Balance) -> Result<Balance> {
            self.get_amount_out(amount_in, self.reserve_b, self.reserve_a)
        }

        #[ink(message)]
        pub fn get_spot_price(&self) -> u128 {
            if self.reserve_a == 0 { return 0; }
            self.reserve_b.saturating_mul(1_000_000).checked_div(self.reserve_a).unwrap_or(0)
        }

        #[ink(message)]
        pub fn get_reserves(&self) -> (Balance, Balance) { (self.reserve_a, self.reserve_b) }

        #[ink(message)]
        pub fn get_lp_balance(&self, account: AccountId) -> Balance { self.lp_balances.get(account).unwrap_or(0) }

        #[ink(message)]
        pub fn get_stats(&self) -> (Balance, Balance, Balance, Balance, Balance) {
            (self.reserve_a, self.reserve_b, self.total_lp, self.volume_a, self.volume_b)
        }

        fn get_amount_out(&self, amount_in: Balance, reserve_in: Balance, reserve_out: Balance) -> Result<Balance> {
            if reserve_in == 0 || reserve_out == 0 { return Err(Error::InsufficientLiquidity); }
            let fee_factor = (10_000u128).saturating_sub(self.fee_bps as u128);
            let in_with_fee = amount_in.saturating_mul(fee_factor);
            let numerator = in_with_fee.saturating_mul(reserve_out);
            let denominator = reserve_in.saturating_mul(10_000).saturating_add(in_with_fee);
            Ok(numerator.checked_div(denominator).unwrap_or(0))
        }

        fn integer_sqrt(&self, n: u128) -> u128 {
            if n == 0 { return 0; }
            let mut x = n;
            let mut y = x.saturating_add(1).checked_div(2).unwrap_or(0);
            while y < x {
                x = y;
                y = x.saturating_add(n.checked_div(x).unwrap_or(0)).checked_div(2).unwrap_or(0);
            }
            x
        }
    }
}