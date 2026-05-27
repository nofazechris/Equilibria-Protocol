#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod price_oracle {

    use ink_storage::{
        traits::SpreadAllocate,
        Mapping,
    };

    /// Price stored as integer with 6 decimal places
    /// e.g. 1_500_000 = $1.50
    pub type Price = u128;

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct PriceOracle {
        /// Contract owner
        owner: AccountId,
        /// Authorized price feeders (keeper wallets)
        feeders: Mapping<AccountId, bool>,
        /// Latest price per asset symbol hash
        prices: Mapping<u32, Price>,
        /// Timestamp of last update per asset
        last_updated: Mapping<u32, u64>,
        /// Simple TWAP: cumulative price × time
        twap_cumulative: Mapping<u32, u128>,
        /// Last TWAP update block
        twap_last_block: Mapping<u32, u32>,
        /// Number of registered feeders
        feeder_count: u32,
    }

    #[ink(event)]
    pub struct PriceUpdated {
        #[ink(topic)]
        asset_id: u32,
        price: Price,
        timestamp: u64,
        feeder: AccountId,
    }

    #[ink(event)]
    pub struct FeederAdded {
        feeder: AccountId,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotOwner,
        NotFeeder,
        AssetNotFound,
        ZeroPrice,
        StalePrice,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    /// Asset IDs — fixed constants for known pairs
    /// POT  = 1
    /// rUSD = 2
    /// USDC = 3
    pub const ASSET_POT:  u32 = 1;
    pub const ASSET_RUSD: u32 = 2;
    pub const ASSET_USDC: u32 = 3;

    impl PriceOracle {

        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                let caller = Self::env().caller();
                contract.owner = caller;
                contract.feeders.insert(caller, &true);
                contract.feeder_count = 1;
            })
        }

        /// Add an authorized price feeder
        #[ink(message)]
        pub fn add_feeder(&mut self, feeder: AccountId) -> Result<()> {
            self.ensure_owner()?;
            self.feeders.insert(feeder, &true);
            self.feeder_count = self.feeder_count.saturating_add(1);
            self.env().emit_event(FeederAdded { feeder });
            Ok(())
        }

        /// Submit a price update for an asset
        #[ink(message)]
        pub fn update_price(&mut self, asset_id: u32, price: Price) -> Result<()> {
            self.ensure_feeder()?;
            if price == 0 {
                return Err(Error::ZeroPrice);
            }

            let now = self.env().block_timestamp();
            let block = self.env().block_number();

            // Update TWAP cumulative
            let last_price = self.prices.get(asset_id).unwrap_or(price);
            let last_block = self.twap_last_block.get(asset_id).unwrap_or(block);
            let blocks_elapsed = block.saturating_sub(last_block) as u128;
            let cumulative = self.twap_cumulative.get(asset_id).unwrap_or(0);
            self.twap_cumulative.insert(
                asset_id,
                &cumulative.saturating_add(last_price.saturating_mul(blocks_elapsed))
            );
            self.twap_last_block.insert(asset_id, &block);

            // Update spot price
            self.prices.insert(asset_id, &price);
            self.last_updated.insert(asset_id, &now);

            self.env().emit_event(PriceUpdated {
                asset_id,
                price,
                timestamp: now,
                feeder: self.env().caller(),
            });
            Ok(())
        }

        /// Get current spot price for an asset
        #[ink(message)]
        pub fn get_price(&self, asset_id: u32) -> Result<Price> {
            self.prices.get(asset_id).ok_or(Error::AssetNotFound)
        }

        /// Get TWAP over last N blocks (simplified)
        #[ink(message)]
        pub fn get_twap(&self, asset_id: u32) -> Result<Price> {
            let current_price = self.prices.get(asset_id).ok_or(Error::AssetNotFound)?;
            let cumulative = self.twap_cumulative.get(asset_id).unwrap_or(0);
            let last_block = self.twap_last_block.get(asset_id).unwrap_or(0);
            let current_block = self.env().block_number();
            let blocks = current_block.saturating_sub(last_block) as u128;

            if blocks == 0 {
                return Ok(current_price);
            }

            let total_cumulative = cumulative
                .saturating_add(current_price.saturating_mul(blocks));
            let total_blocks = (current_block as u128).saturating_add(1);

            Ok(total_cumulative.checked_div(total_blocks).unwrap_or(current_price))
        }

        /// Get price ratio between two assets (asset_a / asset_b) × 1_000_000
        #[ink(message)]
        pub fn get_ratio(&self, asset_a: u32, asset_b: u32) -> Result<u128> {
            let price_a = self.prices.get(asset_a).ok_or(Error::AssetNotFound)?;
            let price_b = self.prices.get(asset_b).ok_or(Error::AssetNotFound)?;
            if price_b == 0 {
                return Err(Error::ZeroPrice);
            }
            Ok(price_a.saturating_mul(1_000_000).checked_div(price_b).unwrap_or(0))
        }

        /// Get last update timestamp
        #[ink(message)]
        pub fn get_last_updated(&self, asset_id: u32) -> u64 {
            self.last_updated.get(asset_id).unwrap_or(0)
        }

        #[ink(message)]
        pub fn get_owner(&self) -> AccountId { self.owner }

        #[ink(message)]
        pub fn is_feeder(&self, account: AccountId) -> bool {
            self.feeders.get(account).unwrap_or(false)
        }

        fn ensure_owner(&self) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }
            Ok(())
        }

        fn ensure_feeder(&self) -> Result<()> {
            if !self.feeders.get(self.env().caller()).unwrap_or(false) {
                return Err(Error::NotFeeder);
            }
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn price_update_works() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            let mut oracle = PriceOracle::new();
            assert!(oracle.update_price(ASSET_POT, 1_500_000).is_ok());
            assert_eq!(oracle.get_price(ASSET_POT).unwrap(), 1_500_000);
        }

        #[ink::test]
        fn ratio_works() {
            let mut oracle = PriceOracle::new();
            oracle.update_price(ASSET_POT,  2_000_000).unwrap();
            oracle.update_price(ASSET_RUSD, 1_000_000).unwrap();
            // POT/rUSD ratio should be 2.0 × 1_000_000 = 2_000_000
            assert_eq!(oracle.get_ratio(ASSET_POT, ASSET_RUSD).unwrap(), 2_000_000);
        }
    }
}