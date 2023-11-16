#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod data;
mod errors;
mod traits;
mod unit_tests;

pub use data::{Id, PSP34Data, PSP34Event};
pub use errors::PSP34Error;
pub use traits::{PSP34Burnable, PSP34Metadata, PSP34Mintable, PSP34};

#[ink::contract]
mod token {
    use crate::{Id, PSP34Burnable, PSP34Data, PSP34Error, PSP34Event, PSP34Mintable, PSP34};

    #[ink(storage)]
    pub struct Token {
        data: PSP34Data,
    }

    impl Token {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                data: PSP34Data::new(),
            }
        }

        fn emit_events(&self, events: ink::prelude::vec::Vec<PSP34Event>) {
            for event in events {
                match event {
                    PSP34Event::Approval {
                        owner,
                        operator,
                        id,
                        approved,
                    } => self.env().emit_event(Approval {
                        owner,
                        operator,
                        id,
                        approved,
                    }),
                    PSP34Event::Transfer { from, to, id } => {
                        self.env().emit_event(Transfer { from, to, id })
                    }
                    PSP34Event::AttributeSet { id, key, data } => {
                        self.env().emit_event(AttributeSet { id, key, data })
                    }
                }
            }
        }
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        operator: AccountId,
        #[ink(topic)]
        id: Option<Id>,
        approved: bool,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        id: Id,
    }

    #[ink(event)]
    pub struct AttributeSet {
        id: Id,
        key: ink::prelude::string::String,
        data: ink::prelude::string::String,
    }

    impl PSP34 for Token {
        #[ink(message)]
        fn collection_id(&self) -> Id {
            self.data.collection_id(self.env().caller())
        }

        #[ink(message)]
        fn total_supply(&self) -> u128 {
            self.data.total_supply()
        }

        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> u128 {
            self.data.balance_of(owner)
        }

        #[ink(message)]
        fn allowance(&self, owner: AccountId, operator: AccountId, id: Option<Id>) -> bool {
            self.data.allowance(owner, operator, id.as_ref())
        }

        #[ink(message)]
        fn transfer(
            &mut self,
            to: AccountId,
            id: Id,
            data: ink::prelude::vec::Vec<u8>,
        ) -> Result<(), PSP34Error> {
            let events = self.data.transfer(self.env().caller(), to, id, data)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn approve(
            &mut self,
            operator: AccountId,
            id: Option<Id>,
            approved: bool,
        ) -> Result<(), PSP34Error> {
            let events = self
                .data
                .approve(self.env().caller(), operator, id, approved)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn owner_of(&self, id: Id) -> Option<AccountId> {
            self.data.owner_of(&id)
        }
    }

    impl PSP34Mintable for Token {
        #[ink(message)]
        fn mint(&mut self, id: Id) -> Result<(), PSP34Error> {
            // Add security, restrict usage of the message
            todo!();
            let events = self.data.mint(self.env().caller(), id)?;
            self.emit_events(events);
            Ok(())
        }
    }

    impl PSP34Burnable for Token {
        #[ink(message)]
        fn burn(&mut self, account: AccountId, id: Id) -> Result<(), PSP34Error> {
            // Add security, restrict usage of the message
            todo!();
            let events = self.data.burn(self.env().caller(), account, id)?;
            self.emit_events(events);
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        crate::tests!(Token, Token::new);
    }
}
