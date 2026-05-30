#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod price_oracle {

    use ink::storage::Mapping;

    pub type Price = u128;
    pub const ASSET_POT:  u32 = 1;
    pub const ASSET_RUSD: u32 = 2;
    pub const ASSET_USDC: u32 = 3;

    #[ink(storage)]
    pub struct PriceOracle {
        owner: AccountId,
        prices: Mapping<u32, Price>,
        last_updated: Mapping<u32, u64>,
        twap_cumulative: Mapping<u32, u128>,
        twap_last_block: Mapping<u32, u32>,
    }

    #[ink(event)]
    pub struct PriceUpdated {
        #[ink(topic)]
        asset_id: u32,
        price: Price,
        feeder: AccountId,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotOwner,
        AssetNotFound,
        ZeroPrice,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl PriceOracle {

        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                owner: Self::env().caller(),
                prices: Mapping::default(),
                last_updated: Mapping::default(),
                twap_cumulative: Mapping::default(),
                twap_last_block: Mapping::default(),
            }
        }

        #[ink(message)]
        pub fn update_price(&mut self, asset_id: u32, price: Price) -> Result<()> {
            self.ensure_owner()?;
            if price == 0 { return Err(Error::ZeroPrice); }
            let now = self.env().block_timestamp();
            let block = self.env().block_number();
            let last_price = self.prices.get(asset_id).unwrap_or(price);
            let last_block = self.twap_last_block.get(asset_id).unwrap_or(block);
            let elapsed = block.saturating_sub(last_block) as u128;
            let cum = self.twap_cumulative.get(asset_id).unwrap_or(0);
            self.twap_cumulative.insert(asset_id, &cum.saturating_add(last_price.saturating_mul(elapsed)));
            self.twap_last_block.insert(asset_id, &block);
            self.prices.insert(asset_id, &price);
            self.last_updated.insert(asset_id, &now);
            self.env().emit_event(PriceUpdated { asset_id, price, feeder: self.env().caller() });
            Ok(())
        }

        #[ink(message)]
        pub fn get_price(&self, asset_id: u32) -> Result<Price> {
            self.prices.get(asset_id).ok_or(Error::AssetNotFound)
        }

        #[ink(message)]
        pub fn get_twap(&self, asset_id: u32) -> Result<Price> {
            let current = self.prices.get(asset_id).ok_or(Error::AssetNotFound)?;
            let cum = self.twap_cumulative.get(asset_id).unwrap_or(0);
            let last_block = self.twap_last_block.get(asset_id).unwrap_or(0);
            let cur_block = self.env().block_number();
            let blocks = cur_block.saturating_sub(last_block) as u128;
            if blocks == 0 { return Ok(current); }
            let total_cum = cum.saturating_add(current.saturating_mul(blocks));
            let total_blocks = (cur_block as u128).saturating_add(1);
            Ok(total_cum.checked_div(total_blocks).unwrap_or(current))
        }

        #[ink(message)]
        pub fn get_ratio(&self, asset_a: u32, asset_b: u32) -> Result<u128> {
            let a = self.prices.get(asset_a).ok_or(Error::AssetNotFound)?;
            let b = self.prices.get(asset_b).ok_or(Error::AssetNotFound)?;
            if b == 0 { return Err(Error::ZeroPrice); }
            Ok(a.saturating_mul(1_000_000).checked_div(b).unwrap_or(0))
        }

        #[ink(message)]
        pub fn get_last_updated(&self, asset_id: u32) -> u64 {
            self.last_updated.get(asset_id).unwrap_or(0)
        }

        #[ink(message)]
        pub fn get_owner(&self) -> AccountId { self.owner }

        fn ensure_owner(&self) -> Result<()> {
            if self.env().caller() != self.owner { return Err(Error::NotOwner); }
            Ok(())
        }
    }
}