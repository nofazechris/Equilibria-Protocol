#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod stability_pool {

    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct StabilityPool {
        owner: AccountId,
        keeper: AccountId,
        total_deposited: Balance,
        total_shares: Balance,
        deployed_capital: Balance,
        yield_accumulated: Balance,
        shares: Mapping<AccountId, Balance>,
        deposits: Mapping<AccountId, Balance>,
        paused: bool,
        max_deploy_pct: u8,
    }

    #[ink(event)]
    pub struct Deposited {
        #[ink(topic)]
        account: AccountId,
        amount: Balance,
        shares_minted: Balance,
    }

    #[ink(event)]
    pub struct Withdrawn {
        #[ink(topic)]
        account: AccountId,
        amount: Balance,
        shares_burned: Balance,
    }

    #[ink(event)]
    pub struct CapitalDeployed { opportunity_id: u64, amount: Balance }

    #[ink(event)]
    pub struct ReturnRecorded { opportunity_id: u64, profit: Balance, loss: Balance }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotOwner, NotKeeper, Paused, ZeroAmount,
        InsufficientShares, ExceedsDeployLimit, InsufficientPoolFunds,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl StabilityPool {

        #[ink(constructor)]
        pub fn new(keeper: AccountId, max_deploy_pct: u8) -> Self {
            Self {
                owner: Self::env().caller(),
                keeper,
                total_deposited: 0,
                total_shares: 0,
                deployed_capital: 0,
                yield_accumulated: 0,
                shares: Mapping::default(),
                deposits: Mapping::default(),
                paused: false,
                max_deploy_pct,
            }
        }

        #[ink(message, payable)]
        pub fn deposit(&mut self) -> Result<Balance> {
            self.ensure_not_paused()?;
            let caller = self.env().caller();
            let amount = self.env().transferred_value();
            if amount == 0 { return Err(Error::ZeroAmount); }
            let shares_to_mint = if self.total_shares == 0 { amount }
                else { amount.saturating_mul(self.total_shares).checked_div(self.total_deposited).unwrap_or(0) };
            let cs = self.shares.get(caller).unwrap_or(0);
            let cd = self.deposits.get(caller).unwrap_or(0);
            self.shares.insert(caller, &(cs.saturating_add(shares_to_mint)));
            self.deposits.insert(caller, &(cd.saturating_add(amount)));
            self.total_deposited = self.total_deposited.saturating_add(amount);
            self.total_shares = self.total_shares.saturating_add(shares_to_mint);
            self.env().emit_event(Deposited { account: caller, amount, shares_minted: shares_to_mint });
            Ok(shares_to_mint)
        }

        #[ink(message)]
        pub fn withdraw(&mut self, share_amount: Balance) -> Result<Balance> {
            self.ensure_not_paused()?;
            let caller = self.env().caller();
            let cs = self.shares.get(caller).unwrap_or(0);
            if share_amount == 0 || share_amount > cs { return Err(Error::InsufficientShares); }
            let withdraw_amount = share_amount.saturating_mul(self.total_deposited).checked_div(self.total_shares).unwrap_or(0);
            self.shares.insert(caller, &(cs.saturating_sub(share_amount)));
            self.total_shares = self.total_shares.saturating_sub(share_amount);
            self.total_deposited = self.total_deposited.saturating_sub(withdraw_amount);
            self.env().transfer(caller, withdraw_amount).map_err(|_| Error::InsufficientPoolFunds)?;
            self.env().emit_event(Withdrawn { account: caller, amount: withdraw_amount, shares_burned: share_amount });
            Ok(withdraw_amount)
        }

        #[ink(message)]
        pub fn record_deployment(&mut self, opportunity_id: u64, amount: Balance) -> Result<()> {
            self.ensure_keeper()?;
            self.ensure_not_paused()?;
            let max_allowed = self.total_deposited.saturating_mul(self.max_deploy_pct as u128).checked_div(100).unwrap_or(0);
            if self.deployed_capital.saturating_add(amount) > max_allowed { return Err(Error::ExceedsDeployLimit); }
            self.deployed_capital = self.deployed_capital.saturating_add(amount);
            self.env().emit_event(CapitalDeployed { opportunity_id, amount });
            Ok(())
        }

        #[ink(message)]
        pub fn record_return(&mut self, opportunity_id: u64, profit: Balance, loss: Balance) -> Result<()> {
            self.ensure_keeper()?;
            if profit > 0 {
                self.yield_accumulated = self.yield_accumulated.saturating_add(profit);
                self.total_deposited = self.total_deposited.saturating_add(profit);
            }
            if loss > 0 {
                self.deployed_capital = self.deployed_capital.saturating_sub(loss);
                self.total_deposited = self.total_deposited.saturating_sub(loss);
            }
            self.deployed_capital = self.deployed_capital.saturating_sub(profit);
            self.env().emit_event(ReturnRecorded { opportunity_id, profit, loss });
            Ok(())
        }

        #[ink(message)] pub fn pause(&mut self) -> Result<()> { self.ensure_owner()?; self.paused = true; Ok(()) }
        #[ink(message)] pub fn unpause(&mut self) -> Result<()> { self.ensure_owner()?; self.paused = false; Ok(()) }
        #[ink(message)] pub fn set_keeper(&mut self, k: AccountId) -> Result<()> { self.ensure_owner()?; self.keeper = k; Ok(()) }
        #[ink(message)] pub fn get_pool_state(&self) -> (Balance, Balance, Balance, Balance, bool) {
            (self.total_deposited, self.total_shares, self.deployed_capital, self.yield_accumulated, self.paused)
        }
        #[ink(message)] pub fn get_shares(&self, a: AccountId) -> Balance { self.shares.get(a).unwrap_or(0) }
        #[ink(message)] pub fn get_deposit(&self, a: AccountId) -> Balance { self.deposits.get(a).unwrap_or(0) }
        #[ink(message)] pub fn get_owner(&self) -> AccountId { self.owner }
        #[ink(message)] pub fn get_keeper(&self) -> AccountId { self.keeper }
        #[ink(message)] pub fn is_paused(&self) -> bool { self.paused }

        fn ensure_owner(&self) -> Result<()> { if self.env().caller() != self.owner { return Err(Error::NotOwner); } Ok(()) }
        fn ensure_keeper(&self) -> Result<()> { if self.env().caller() != self.keeper { return Err(Error::NotKeeper); } Ok(()) }
        fn ensure_not_paused(&self) -> Result<()> { if self.paused { return Err(Error::Paused); } Ok(()) }
    }
}