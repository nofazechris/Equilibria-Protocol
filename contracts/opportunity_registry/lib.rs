#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod opportunity_registry {
    use ink_prelude::{string::String, vec::Vec};
    use ink_storage::Mapping;

    pub type RegistryResult<T> = core::result::Result<T, RegistryError>;

    const MAX_PAIR_LEN: usize = 64;
    const MAX_PAGE_SIZE: u32 = 100;

    #[derive(scale::Encode, scale::Decode, Clone, Copy, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum OpportunityStatus {
        Posted,
        Executing,
        Closed,
        Expired,
    }

    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Opportunity {
        pub id: u64,
        pub posted_by: AccountId,
        pub pair: String,
        pub spread_bps: u32,
        pub profitability: u8,
        pub risk: u8,
        pub stability_impact: u8,
        pub confidence: u8,
        pub composite_score: u8,
        pub recommended_size: Balance,
        pub status: OpportunityStatus,
        pub created_at: Timestamp,
        pub updated_at: Timestamp,
        pub executed_at: Option<Timestamp>,
        pub closed_at: Option<Timestamp>,
        pub profit: Option<Balance>,
        pub loss: Option<Balance>,
    }

    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct RegistryState {
        pub owner: AccountId,
        pub keeper: AccountId,
        pub paused: bool,
        pub min_composite_score: u8,
        pub next_id: u64,
        pub total_opportunities: u64,
        pub active_opportunities: u64,
        pub closed_opportunities: u64,
        pub expired_opportunities: u64,
    }

    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum RegistryError {
        NotOwner,
        NotKeeper,
        Paused,
        InvalidInput,
        InvalidScore,
        InvalidStatus,
        OpportunityNotFound,
        OpportunityAlreadyExists,
        Overflow,
        InconsistentState,
    }

    #[ink(event)]
    pub struct OpportunityPosted {
        #[ink(topic)]
        pub id: u64,
        #[ink(topic)]
        pub posted_by: AccountId,
        pub pair: String,
        pub spread_bps: u32,
        pub profitability: u8,
        pub risk: u8,
        pub stability_impact: u8,
        pub confidence: u8,
        pub composite_score: u8,
        pub recommended_size: Balance,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct OpportunityStatusChanged {
        #[ink(topic)]
        pub id: u64,
        #[ink(topic)]
        pub changed_by: AccountId,
        pub from: OpportunityStatus,
        pub to: OpportunityStatus,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct KeeperUpdated {
        #[ink(topic)]
        pub old_keeper: AccountId,
        #[ink(topic)]
        pub new_keeper: AccountId,
    }

    #[ink(event)]
    pub struct OwnershipTransferred {
        #[ink(topic)]
        pub old_owner: AccountId,
        #[ink(topic)]
        pub new_owner: AccountId,
    }

    #[ink(event)]
    pub struct PauseToggled {
        #[ink(topic)]
        pub paused_by: AccountId,
        pub paused: bool,
    }

    #[ink(storage)]
    pub struct OpportunityRegistry {
        owner: AccountId,
        keeper: AccountId,
        paused: bool,
        min_composite_score: u8,
        next_id: u64,
        opportunities: Mapping<u64, Opportunity>,
        all_ids: Vec<u64>,
        active_ids: Vec<u64>,
        active_index: Mapping<u64, u64>,
        counts: RegistryCounts,
    }

    #[derive(Default)]
    struct RegistryCounts {
        active: u64,
        closed: u64,
        expired: u64,
    }

    impl OpportunityRegistry {
        #[ink(constructor)]
        pub fn new(keeper: AccountId, min_composite_score: u8) -> Self {
            let caller = Self::env().caller();
            Self::new_with_owner(caller, keeper, min_composite_score)
        }

        #[ink(constructor)]
        pub fn new_with_owner(
            owner: AccountId,
            keeper: AccountId,
            min_composite_score: u8,
        ) -> Self {
            assert!(min_composite_score <= 100, "min score must be 0..=100");
            Self {
                owner,
                keeper,
                paused: false,
                min_composite_score,
                next_id: 1,
                opportunities: Mapping::default(),
                all_ids: Vec::new(),
                active_ids: Vec::new(),
                active_index: Mapping::default(),
                counts: RegistryCounts::default(),
            }
        }

        #[ink(message)]
        pub fn get_state(&self) -> RegistryState {
            RegistryState {
                owner: self.owner,
                keeper: self.keeper,
                paused: self.paused,
                min_composite_score: self.min_composite_score,
                next_id: self.next_id,
                total_opportunities: self.all_ids.len() as u64,
                active_opportunities: self.counts.active,
                closed_opportunities: self.counts.closed,
                expired_opportunities: self.counts.expired,
            }
        }

        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        #[ink(message)]
        pub fn keeper(&self) -> AccountId {
            self.keeper
        }

        #[ink(message)]
        pub fn paused(&self) -> bool {
            self.paused
        }

        #[ink(message)]
        pub fn min_composite_score(&self) -> u8 {
            self.min_composite_score
        }

        #[ink(message)]
        pub fn get_opportunity(&self, id: u64) -> Option<Opportunity> {
            self.opportunities.get(id)
        }

        #[ink(message)]
        pub fn get_active_opportunities(&self, limit: u32, offset: u32) -> Vec<Opportunity> {
            self.paginate_and_fetch(&self.active_ids, limit, offset)
        }

        #[ink(message)]
        pub fn get_historical_opportunities(&self, limit: u32, offset: u32) -> Vec<Opportunity> {
            let mut historical_ids: Vec<u64> = Vec::new();
            let start = offset as usize;
            let end = core::cmp::min(self.all_ids.len(), start.saturating_add(limit.min(MAX_PAGE_SIZE) as usize));
            for id in self.all_ids.iter().skip(start).take(end.saturating_sub(start)) {
                if let Some(opportunity) = self.opportunities.get(*id) {
                    match opportunity.status {
                        OpportunityStatus::Closed | OpportunityStatus::Expired => historical_ids.push(*id),
                        _ => {}
                    }
                }
            }
            self.fetch_by_ids(&historical_ids)
        }

        #[ink(message)]
        pub fn post_opportunity(
            &mut self,
            pair: String,
            spread_bps: u32,
            profitability: u8,
            risk: u8,
            stability_impact: u8,
            confidence: u8,
            composite_score: u8,
            recommended_size: Balance,
        ) -> RegistryResult<u64> {
            self.ensure_not_paused()?;
            self.ensure_keeper()?;
            self.validate_opportunity_input(
                &pair,
                spread_bps,
                profitability,
                risk,
                stability_impact,
                confidence,
                composite_score,
                recommended_size,
            )?;

            if composite_score < self.min_composite_score {
                return Err(RegistryError::InvalidScore);
            }

            let id = self.next_id;
            self.next_id = self.next_id.checked_add(1).ok_or(RegistryError::Overflow)?;

            let now = self.env().block_timestamp();
            let caller = self.env().caller();

            let opportunity = Opportunity {
                id,
                posted_by: caller,
                pair,
                spread_bps,
                profitability,
                risk,
                stability_impact,
                confidence,
                composite_score,
                recommended_size,
                status: OpportunityStatus::Posted,
                created_at: now,
                updated_at: now,
                executed_at: None,
                closed_at: None,
                profit: None,
                loss: None,
            };

            if self.opportunities.get(id).is_some() {
                return Err(RegistryError::OpportunityAlreadyExists);
            }

            self.opportunities.insert(id, &opportunity);
            self.all_ids.push(id);
            self.push_active_id(id)?;

            self.env().emit_event(OpportunityPosted {
                id,
                posted_by: caller,
                pair: opportunity.pair.clone(),
                spread_bps: opportunity.spread_bps,
                profitability: opportunity.profitability,
                risk: opportunity.risk,
                stability_impact: opportunity.stability_impact,
                confidence: opportunity.confidence,
                composite_score: opportunity.composite_score,
                recommended_size: opportunity.recommended_size,
                timestamp: now,
            });

            Ok(id)
        }

        #[ink(message)]
        pub fn execute_opportunity(&mut self, id: u64) -> RegistryResult<()> {
            self.ensure_not_paused()?;
            self.ensure_keeper()?;

            let mut opportunity = self.load_opportunity(id)?;
            if opportunity.status != OpportunityStatus::Posted {
                return Err(RegistryError::InvalidStatus);
            }

            let now = self.env().block_timestamp();
            let caller = self.env().caller();
            let from = opportunity.status;

            opportunity.status = OpportunityStatus::Executing;
            opportunity.executed_at = Some(now);
            opportunity.updated_at = now;
            self.opportunities.insert(id, &opportunity);

            self.env().emit_event(OpportunityStatusChanged {
                id,
                changed_by: caller,
                from,
                to: OpportunityStatus::Executing,
                timestamp: now,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn close_opportunity(
            &mut self,
            id: u64,
            profit: Balance,
            loss: Balance,
        ) -> RegistryResult<()> {
            self.ensure_keeper()?;

            if profit > 0 && loss > 0 {
                return Err(RegistryError::InvalidInput);
            }

            let mut opportunity = self.load_opportunity(id)?;
            if opportunity.status != OpportunityStatus::Executing {
                return Err(RegistryError::InvalidStatus);
            }

            let now = self.env().block_timestamp();
            let caller = self.env().caller();
            let from = opportunity.status;

            opportunity.status = OpportunityStatus::Closed;
            opportunity.closed_at = Some(now);
            opportunity.updated_at = now;
            opportunity.profit = Some(profit);
            opportunity.loss = Some(loss);
            self.opportunities.insert(id, &opportunity);

            self.remove_active_id(id)?;

            self.counts.active = self.counts.active.saturating_sub(1);
            self.counts.closed = self.counts.closed.saturating_add(1);

            self.env().emit_event(OpportunityStatusChanged {
                id,
                changed_by: caller,
                from,
                to: OpportunityStatus::Closed,
                timestamp: now,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn expire_opportunity(&mut self, id: u64) -> RegistryResult<()> {
            self.ensure_keeper()?;

            let mut opportunity = self.load_opportunity(id)?;
            match opportunity.status {
                OpportunityStatus::Posted | OpportunityStatus::Executing => {}
                _ => return Err(RegistryError::InvalidStatus),
            }

            let now = self.env().block_timestamp();
            let caller = self.env().caller();
            let from = opportunity.status;

            opportunity.status = OpportunityStatus::Expired;
            opportunity.closed_at = Some(now);
            opportunity.updated_at = now;
            self.opportunities.insert(id, &opportunity);

            self.remove_active_id(id)?;

            self.counts.active = self.counts.active.saturating_sub(1);
            self.counts.expired = self.counts.expired.saturating_add(1);

            self.env().emit_event(OpportunityStatusChanged {
                id,
                changed_by: caller,
                from,
                to: OpportunityStatus::Expired,
                timestamp: now,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn pause(&mut self) -> RegistryResult<()> {
            self.ensure_owner()?;
            self.paused = true;
            self.env().emit_event(PauseToggled {
                paused_by: self.env().caller(),
                paused: true,
            });
            Ok(())
        }

        #[ink(message)]
        pub fn unpause(&mut self) -> RegistryResult<()> {
            self.ensure_owner()?;
            self.paused = false;
            self.env().emit_event(PauseToggled {
                paused_by: self.env().caller(),
                paused: false,
            });
            Ok(())
        }

        #[ink(message)]
        pub fn set_keeper(&mut self, new_keeper: AccountId) -> RegistryResult<()> {
            self.ensure_owner()?;
            let old_keeper = self.keeper;
            self.keeper = new_keeper;
            self.env().emit_event(KeeperUpdated {
                old_keeper,
                new_keeper,
            });
            Ok(())
        }

        #[ink(message)]
        pub fn set_min_composite_score(&mut self, new_min: u8) -> RegistryResult<()> {
            self.ensure_owner()?;
            if new_min > 100 {
                return Err(RegistryError::InvalidScore);
            }
            self.min_composite_score = new_min;
            Ok(())
        }

        #[ink(message)]
        pub fn transfer_ownership(&mut self, new_owner: AccountId) -> RegistryResult<()> {
            self.ensure_owner()?;
            let old_owner = self.owner;
            self.owner = new_owner;
            self.env().emit_event(OwnershipTransferred {
                old_owner,
                new_owner,
            });
            Ok(())
        }

        #[ink(message)]
        pub fn is_active(&self, id: u64) -> bool {
            match self.opportunities.get(id) {
                Some(opportunity) => matches!(
                    opportunity.status,
                    OpportunityStatus::Posted | OpportunityStatus::Executing
                ),
                None => false,
            }
        }

        #[ink(message)]
        pub fn active_count(&self) -> u64 {
            self.counts.active
        }

        #[ink(message)]
        pub fn historical_count(&self) -> u64 {
            self.counts.closed + self.counts.expired
        }

        fn ensure_owner(&self) -> RegistryResult<()> {
            if self.env().caller() != self.owner {
                return Err(RegistryError::NotOwner);
            }
            Ok(())
        }

        fn ensure_keeper(&self) -> RegistryResult<()> {
            if self.env().caller() != self.keeper {
                return Err(RegistryError::NotKeeper);
            }
            Ok(())
        }

        fn ensure_not_paused(&self) -> RegistryResult<()> {
            if self.paused {
                return Err(RegistryError::Paused);
            }
            Ok(())
        }

        fn validate_opportunity_input(
            &self,
            pair: &String,
            spread_bps: u32,
            profitability: u8,
            risk: u8,
            stability_impact: u8,
            confidence: u8,
            composite_score: u8,
            recommended_size: Balance,
        ) -> RegistryResult<()> {
            if pair.is_empty() || pair.len() > MAX_PAIR_LEN {
                return Err(RegistryError::InvalidInput);
            }
            if spread_bps == 0 {
                return Err(RegistryError::InvalidInput);
            }
            if profitability > 100
                || risk > 100
                || stability_impact > 100
                || confidence > 100
                || composite_score > 100
            {
                return Err(RegistryError::InvalidScore);
            }
            if recommended_size == 0 {
                return Err(RegistryError::InvalidInput);
            }
            Ok(())
        }

        fn load_opportunity(&self, id: u64) -> RegistryResult<Opportunity> {
            self.opportunities
                .get(id)
                .ok_or(RegistryError::OpportunityNotFound)
        }

        fn push_active_id(&mut self, id: u64) -> RegistryResult<()> {
            if self.active_index.get(id).is_some() {
                return Err(RegistryError::OpportunityAlreadyExists);
            }
            let index = self.active_ids.len() as u64;
            self.active_ids.push(id);
            self.active_index.insert(id, &index);
            self.counts.active = self.counts.active.saturating_add(1);
            Ok(())
        }

        fn remove_active_id(&mut self, id: u64) -> RegistryResult<()> {
            let index = self
                .active_index
                .get(id)
                .ok_or(RegistryError::InconsistentState)? as usize;

            if index >= self.active_ids.len() {
                return Err(RegistryError::InconsistentState);
            }

            let last_index = self.active_ids.len() - 1;
            self.active_ids.swap_remove(index);

            if index < self.active_ids.len() {
                let moved_id = self.active_ids[index];
                self.active_index.insert(moved_id, &(index as u64));
            }

            if id != self.active_ids.get(index).copied().unwrap_or(id) && index == last_index {
                // no-op: when removing the last element, the slot is simply gone
            }

            self.active_index.remove(id);
            Ok(())
        }

        fn paginate_and_fetch(
            &self,
            ids: &Vec<u64>,
            limit: u32,
            offset: u32,
        ) -> Vec<Opportunity> {
            let limit = limit.min(MAX_PAGE_SIZE) as usize;
            let start = offset as usize;
            if start >= ids.len() || limit == 0 {
                return Vec::new();
            }
            let end = core::cmp::min(ids.len(), start.saturating_add(limit));
            let mut out = Vec::new();
            for id in ids.iter().skip(start).take(end.saturating_sub(start)) {
                if let Some(opportunity) = self.opportunities.get(*id) {
                    out.push(opportunity);
                }
            }
            out
        }

        fn fetch_by_ids(&self, ids: &Vec<u64>) -> Vec<Opportunity> {
            let mut out = Vec::new();
            for id in ids.iter() {
                if let Some(opportunity) = self.opportunities.get(*id) {
                    out.push(opportunity);
                }
            }
            out
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_env::test;

        fn default_accounts() -> test::DefaultAccounts<ink_env::DefaultEnvironment> {
            test::default_accounts::<ink_env::DefaultEnvironment>()
        }

        fn set_caller(sender: AccountId) {
            test::set_caller::<ink_env::DefaultEnvironment>(sender);
        }

        fn set_block_time(ts: u64) {
            test::set_block_timestamp::<ink_env::DefaultEnvironment>(ts);
        }

        #[ink::test]
        fn owner_initializes_registry() {
            let accounts = default_accounts();
            set_caller(accounts.alice);
            let registry = OpportunityRegistry::new(accounts.bob, 70);
            assert_eq!(registry.owner(), accounts.alice);
            assert_eq!(registry.keeper(), accounts.bob);
            assert!(!registry.paused());
            assert_eq!(registry.min_composite_score(), 70);
        }

        #[ink::test]
        fn keeper_can_post_execute_and_close() {
            let accounts = default_accounts();
            let mut registry = OpportunityRegistry::new(accounts.bob, 70);

            set_caller(accounts.bob);
            set_block_time(1000);

            let id = registry
                .post_opportunity(
                    String::from("POT/rUSD"),
                    234,
                    87,
                    23,
                    91,
                    78,
                    84,
                    18_600,
                )
                .expect("post should succeed");

            assert_eq!(id, 1);
            assert_eq!(registry.active_count(), 1);
            assert!(registry.is_active(id));

            registry.execute_opportunity(id).expect("execute should succeed");
            let opp = registry.get_opportunity(id).expect("exists");
            assert_eq!(opp.status, OpportunityStatus::Executing);
            assert_eq!(opp.executed_at, Some(1000));

            registry.close_opportunity(id, 2_000, 0).expect("close should succeed");
            let opp = registry.get_opportunity(id).expect("exists");
            assert_eq!(opp.status, OpportunityStatus::Closed);
            assert_eq!(opp.closed_at, Some(1000));
            assert_eq!(opp.profit, Some(2_000));
            assert_eq!(opp.loss, Some(0));
            assert_eq!(registry.active_count(), 0);
            assert_eq!(registry.historical_count(), 1);
        }

        #[ink::test]
        fn non_keeper_cannot_post() {
            let accounts = default_accounts();
            let mut registry = OpportunityRegistry::new(accounts.bob, 70);

            set_caller(accounts.charlie);
            set_block_time(1000);

            let err = registry
                .post_opportunity(
                    String::from("POT/rUSD"),
                    234,
                    87,
                    23,
                    91,
                    78,
                    84,
                    18_600,
                )
                .expect_err("should fail");
            assert_eq!(err, RegistryError::NotKeeper);
        }

        #[ink::test]
        fn owner_can_update_keeper() {
            let accounts = default_accounts();
            let mut registry = OpportunityRegistry::new(accounts.bob, 70);

            set_caller(accounts.alice);
            registry.set_keeper(accounts.charlie).expect("owner can update keeper");
            assert_eq!(registry.keeper(), accounts.charlie);
        }

        #[ink::test]
        fn pause_blocks_posting() {
            let accounts = default_accounts();
            let mut registry = OpportunityRegistry::new(accounts.bob, 70);

            set_caller(accounts.alice);
            registry.pause().expect("owner can pause");

            set_caller(accounts.bob);
            set_block_time(1000);

            let err = registry
                .post_opportunity(
                    String::from("POT/rUSD"),
                    234,
                    87,
                    23,
                    91,
                    78,
                    84,
                    18_600,
                )
                .expect_err("paused registry should reject posting");
            assert_eq!(err, RegistryError::Paused);
        }
    }
}
