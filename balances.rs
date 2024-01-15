#[cfg(not(feature = "enumerable"))]
pub mod balance_manager {
    use crate::{data::Id, PSP34Error};
    use ink::{primitives::AccountId, storage::Mapping};
    use ink::prelude::string::String;

    #[ink::storage_item]
    #[derive(Default, Debug)]
    pub struct Balances {
        owned_tokens_count: Mapping<AccountId, u32>,
        total_supply: u128,
    }

    impl Balances {
        pub fn new() -> Balances {
            Default::default()
        }

        pub fn balance_of(&self, owner: &AccountId) -> u32 {
            self.owned_tokens_count.get(owner).unwrap_or(0)
        }

        pub fn increase_balance(
            &mut self,
            owner: &AccountId,
            _id: &Id,
            increase_supply: bool,
        ) -> Result<(), PSP34Error> {
            let mut to_balance = self.balance_of(owner);
            to_balance = to_balance
                .checked_add(1)
                .ok_or(PSP34Error::Custom(String::from(
                    "Max PSP34 balance exceeded. Max balance limited to 2^32-1.",
                )))?;
            self.owned_tokens_count.insert(owner, &to_balance);

            if increase_supply {
                self.total_supply =
                    self.total_supply
                        .checked_add(1)
                        .ok_or(PSP34Error::Custom(String::from(
                            "Max PSP34 supply exceeded. Max supply limited to 2^128-1.",
                        )))?;
            }

            Ok(())
        }

        pub fn decrease_balance(&mut self, owner: &AccountId, _id: &Id, decrease_supply: bool) {
            let from_balance = self.balance_of(owner);
            if from_balance <= 1 {
                self.owned_tokens_count.remove(owner);
            } else {
                self.owned_tokens_count.insert(owner, &(from_balance - 1));
            }
            if decrease_supply {
                self.total_supply -= 1;
            }
        }

        pub fn total_supply(&self) -> u128 {
            self.total_supply
        }
    }
}

#[cfg(feature = "enumerable")]
pub mod balance_manager {
    use crate::{data::Id, PSP34Error};
    use ink::{prelude::vec::Vec, primitives::AccountId, storage::Mapping};

    #[ink::storage_item]
    #[derive(Default, Debug)]
    pub struct Balances {
        enumerable: Mapping<Option<AccountId>, Vec<Id>>,
    }

    impl Balances {
        pub fn new() -> Balances {
            Default::default()
        }

        pub fn owners_token_by_index(
            &self,
            owner: AccountId,
            index: u128,
        ) -> Result<Id, PSP34Error> {
            self._get_value(&Some(owner), index)
                .ok_or(PSP34Error::TokenNotExists)
        }

        pub fn token_by_index(&self, index: u128) -> Result<Id, PSP34Error> {
            self._get_value(&None, index)
                .ok_or(PSP34Error::TokenNotExists)
        }

        fn _get_value(&self, key: &Option<AccountId>, index: u128) -> Option<Id> {
            self.enumerable
                .get(key)
                .and_then(|values| values.get(usize::try_from(index).unwrap()).cloned())
        }

        fn _insert(&mut self, key: &Option<AccountId>, value: &Id) {
            let mut values = self.enumerable.get(key).unwrap_or_default();
            values.push(value.clone());
            self.enumerable.insert(key, &values);
        }

        fn _remove(&mut self, key: &Option<AccountId>, value: &Id) {
            if let Some(mut values) = self.enumerable.get(key) {
                if let Some(pos) = values.iter().position(|v| v == value) {
                    values.swap_remove(pos);
                    self.enumerable.insert(key, &values);
                }
            }
        }

        fn _count(&self, key: &Option<AccountId>) -> u128 {
            self.enumerable
                .get(key)
                .map_or(0, |values| values.len())
                .try_into()
                .unwrap()
        }

        pub fn balance_of(&self, owner: &AccountId) -> u32 {
            self._count(&Some(*owner)) as u32
        }

        pub fn increase_balance(
            &mut self,
            owner: &AccountId,
            id: &Id,
            increase_supply: bool,
        ) -> Result<(), PSP34Error> {
            self._insert(&Some(*owner), id);
            if increase_supply {
                self._insert(&None, id);
            }

            Ok(())
        }

        pub fn decrease_balance(&mut self, owner: &AccountId, id: &Id, decrease_supply: bool) {              
            self._remove(&Some(*owner), id);
            if self.balance_of(owner) == 0 {
                self.enumerable.remove(Some(owner));
            }
            if decrease_supply {
                self._remove(&None, id);
            }
        }

        pub fn total_supply(&self) -> u128 {
            self._count(&None)
        }
    }
}
