#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod opportunity_registry {

    use ink::storage::Mapping;

    #[derive(scale::Encode, scale::Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Opportunity {
        pair: [u8; 12],
        profit_score: u8,
        risk_score: u8,
        stability_score: u8,
        confidence_score: u8,
        composite: u8,
        size: u128,
        status: u8, // 0=active 1=executed 2=rejected
        block_number: u32,
    }

    #[ink(storage)]
    pub struct OpportunityRegistry {
        owner: AccountId,
        keeper: AccountId,
        opportunities: Mapping<u64, Opportunity>,
        opportunity_count: u64,
        total_executed: u64,
        total_rejected: u64,
        paused: bool,
    }

    #[ink(event)]
    pub struct OpportunityPosted {
        #[ink(topic)]
        id: u64,
        composite: u8,
    }

    #[ink(event)]
    pub struct OpportunityExecuted {
        #[ink(topic)]
        id: u64,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotOwner,
        NotKeeper,
        Paused,
        NotFound,
        AlreadyClosed,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl OpportunityRegistry {

        #[ink(constructor)]
        pub fn new(keeper: AccountId) -> Self {
            Self {
                owner: Self::env().caller(),
                keeper,
                opportunities: Mapping::default(),
                opportunity_count: 0,
                total_executed: 0,
                total_rejected: 0,
                paused: false,
            }
        }

        #[ink(message)]
        pub fn post_opportunity(
            &mut self,
            pair: [u8; 12],
            profit_score: u8,
            risk_score: u8,
            stability_score: u8,
            confidence_score: u8,
            composite: u8,
            size: u128,
        ) -> Result<u64> {
            self.ensure_keeper()?;
            self.ensure_not_paused()?;

            let id = self.opportunity_count.saturating_add(1);
            let opp = Opportunity {
                pair,
                profit_score,
                risk_score,
                stability_score,
                confidence_score,
                composite,
                size,
                status: 0,
                block_number: self.env().block_number(),
            };

            self.opportunities.insert(id, &opp);
            self.opportunity_count = id;

            self.env().emit_event(OpportunityPosted { id, composite });
            Ok(id)
        }

        #[ink(message)]
        pub fn execute_opportunity(&mut self, id: u64) -> Result<()> {
            self.ensure_keeper()?;
            let mut opp = self.opportunities.get(id).ok_or(Error::NotFound)?;
            if opp.status != 0 { return Err(Error::AlreadyClosed); }
            opp.status = 1;
            self.opportunities.insert(id, &opp);
            self.total_executed = self.total_executed.saturating_add(1);
            self.env().emit_event(OpportunityExecuted { id });
            Ok(())
        }

        #[ink(message)]
        pub fn reject_opportunity(&mut self, id: u64) -> Result<()> {
            self.ensure_keeper()?;
            let mut opp = self.opportunities.get(id).ok_or(Error::NotFound)?;
            if opp.status != 0 { return Err(Error::AlreadyClosed); }
            opp.status = 2;
            self.opportunities.insert(id, &opp);
            self.total_rejected = self.total_rejected.saturating_add(1);
            Ok(())
        }

        #[ink(message)]
        pub fn get_opportunity(&self, id: u64) -> Option<(u8, u8, u8, u8, u8, u128, u8, u32)> {
            self.opportunities.get(id).map(|o| (
                o.profit_score, o.risk_score, o.stability_score,
                o.confidence_score, o.composite, o.size, o.status, o.block_number
            ))
        }

        #[ink(message)]
        pub fn get_stats(&self) -> (u64, u64, u64) {
            (self.opportunity_count, self.total_executed, self.total_rejected)
        }

        #[ink(message)]
        pub fn get_count(&self) -> u64 { self.opportunity_count }

        #[ink(message)]
        pub fn get_owner(&self) -> AccountId { self.owner }

        #[ink(message)]
        pub fn get_keeper(&self) -> AccountId { self.keeper }

        #[ink(message)]
        pub fn pause(&mut self) -> Result<()> { self.ensure_owner()?; self.paused = true; Ok(()) }

        #[ink(message)]
        pub fn unpause(&mut self) -> Result<()> { self.ensure_owner()?; self.paused = false; Ok(()) }

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
}