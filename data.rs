use crate::PSP34Error;
use ink::{
    prelude::{string::String, vec, vec::Vec},
    primitives::AccountId,
    storage::Mapping,
};

#[cfg(feature = "std")]
use ink::storage::traits::StorageLayout;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub enum Id {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Bytes(Vec<u8>),
}

pub enum PSP34Event {
    Transfer {
        from: Option<AccountId>,
        to: Option<AccountId>,
        id: Id,
    },
    Approval {
        owner: AccountId,
        operator: AccountId,
        id: Option<Id>,
        approved: bool,
    },
    AttributeSet {
        id: Id,
        key: String,
        data: String,
    },
}

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct PSP34Data {
    token_owner: Mapping<Id, AccountId>,
    owned_tokens_count: Mapping<AccountId, u128>,
    operator_approvals: Mapping<(AccountId, AccountId, Option<Id>), ()>,
    total_supply: u128,
}

impl PSP34Data {
    pub fn new() -> PSP34Data {
        Default::default()
    }

    pub fn total_supply(&self) -> u128 {
        self.total_supply
    }

    pub fn balance_of(&self, owner: AccountId) -> u128 {
        self.owned_tokens_count.get(owner).unwrap_or_default()
    }

    pub fn owner_of(&self, id: &Id) -> Option<AccountId> {
        self.token_owner.get(id)
    }

    pub fn allowance(&self, owner: AccountId, operator: AccountId, id: Option<&Id>) -> bool {
        self.operator_approvals
            .get((owner, operator, &None))
            .is_some()
            || id.is_some() && self.operator_approvals.get((owner, operator, id)).is_some()
    }

    pub fn collection_id(&self, account_id: AccountId) -> Id {
        Id::Bytes(<_ as AsRef<[u8; 32]>>::as_ref(&account_id).to_vec())
    }

    pub fn approve(
        &mut self,
        mut caller: AccountId,
        operator: AccountId,
        id: Option<Id>,
        approved: bool,
    ) -> Result<Vec<PSP34Event>, PSP34Error> {
        if let Some(id) = &id {
            let owner = self.owner_of(id).ok_or(PSP34Error::TokenNotExists)?;
            if approved && owner == operator {
                return Err(PSP34Error::SelfApprove);
            }

            if owner != caller && !self.allowance(owner, caller, None) {
                return Err(PSP34Error::NotApproved);
            }

            if !approved && self.allowance(owner, operator, None) {
                return Err(PSP34Error::Custom(String::from(
                    "Cannot revoke approval for a single token, when the operator has approval for all tokens."
                )));
            }
            caller = owner;
        }

        if approved {
            self.operator_approvals
                .insert((caller, operator, id.as_ref()), &());
        } else {
            self.operator_approvals
                .remove((caller, operator, id.as_ref()));
        }

        Ok(vec![PSP34Event::Approval {
            owner: caller,
            operator,
            id,
            approved,
        }])
    }

    pub fn transfer(
        &mut self,
        caller: AccountId,
        to: AccountId,
        id: Id,
        _data: Vec<u8>,
    ) -> Result<Vec<PSP34Event>, PSP34Error> {
        let owner = self.owner_of(&id).ok_or(PSP34Error::TokenNotExists)?;

        if owner == to {
            return Ok(vec![]);
        }

        if owner != caller && !self.allowance(owner, caller, Some(&id)) {
            return Err(PSP34Error::NotApproved);
        }

        let from_balance = self.balance_of(owner);
        if from_balance == 1 {
            self.owned_tokens_count.remove(owner);
        } else {
            self.owned_tokens_count
                .insert(owner, &(from_balance.checked_sub(1).unwrap()));
        }
        self.operator_approvals.remove((owner, caller, Some(&id)));
        self.token_owner.remove(&id);

        self.token_owner.insert(&id, &to);
        let to_balance = self.balance_of(to);
        self.owned_tokens_count
            .insert(to, &(to_balance.checked_add(1).unwrap()));

        Ok(vec![PSP34Event::Transfer {
            from: Some(caller),
            to: Some(to),
            id,
        }])
    }

    pub fn mint(&mut self, account: AccountId, id: Id) -> Result<Vec<PSP34Event>, PSP34Error> {
        if self.owner_of(&id).is_some() {
            return Err(PSP34Error::TokenExists);
        }
        let new_supply =
            self.total_supply
                .checked_add(1)
                .ok_or(PSP34Error::Custom(String::from(
                    "Max PSP34 supply exceeded. Max supply limited to 2^128-1.",
                )))?;
        self.total_supply = new_supply;
        let new_balance = self.balance_of(account).saturating_add(1);
        self.owned_tokens_count.insert(account, &new_balance);
        self.token_owner.insert(&id, &account);

        Ok(vec![PSP34Event::Transfer {
            from: None,
            to: Some(account),
            id,
        }])
    }

    pub fn burn(
        &mut self,
        caller: AccountId,
        account: AccountId,
        id: Id,
    ) -> Result<Vec<PSP34Event>, PSP34Error> {
        if self.owner_of(&id).is_none() {
            return Err(PSP34Error::TokenNotExists);
        }
        if account != caller && !self.allowance(caller, account, None) {
            return Err(PSP34Error::NotApproved);
        }
        self.token_owner.remove(&id);
        let new_balance = self.balance_of(account).saturating_sub(1);
        if new_balance == 0 {
            self.owned_tokens_count.remove(account);
        } else {
            self.owned_tokens_count.insert(account, &new_balance);
        }
        self.total_supply = self.total_supply.saturating_sub(1);

        Ok(vec![PSP34Event::Transfer {
            from: Some(account),
            to: None,
            id,
        }])
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::U8(0)
    }
}
