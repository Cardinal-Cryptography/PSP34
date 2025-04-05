#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod balances;
mod data;
mod errors;
pub mod metadata;
mod traits;
mod unit_tests;

pub use data::{Id, PSP34Data, PSP34Event};
pub use errors::PSP34Error;
pub use traits::{PSP34Burnable, PSP34Metadata, PSP34Mintable, PSP34};

#[cfg(feature = "enumerable")]
pub use traits::PSP34Enumerable;

// An example code of a smart contract using PSP34Data struct to implement
// the functionality of PSP34 fungible token.
//
// Any contract can be easily enriched to act as PSP34 token by:
// (1) adding PSP34Data to contract storage
// (2) properly initializing it
// (3) defining the correct AttributeSet, Transfer and Approval events
// (4) implementing PSP34 trait based on PSP34Data methods
// (5) properly emitting resulting events
//
// Implemented the optional PSP34Mintable (6), PSP34Burnable (7), and PSP34Metadata (8) extensions
// and included unit tests (8).

#[ink::contract]
mod token {
    use crate::{
        metadata, Id, PSP34Burnable, PSP34Data, PSP34Error, PSP34Event, PSP34Metadata,
        PSP34Mintable, PSP34,
    };
    use ink::prelude::vec::Vec;

    #[cfg(feature = "enumerable")]
    use crate::PSP34Enumerable;

    #[ink(storage)]
    pub struct Token {
        data: PSP34Data,          // (1)
        metadata: metadata::Data, // (8)
    }

    impl Token {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                data: PSP34Data::new(),              // (2)
                metadata: metadata::Data::default(), // (8)
            }
        }

        // A helper function translating a vector of PSP34Events into the proper
        // ink event types (defined internally in this contract) and emitting them.
        // (5)
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

    // (3)
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

    // (3)
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        id: Id,
    }

    // (3)
    #[ink(event)]
    pub struct AttributeSet {
        id: Id,
        key: Vec<u8>,
        data: Vec<u8>,
    }

    // (4)
    impl PSP34 for Token {
        #[ink(message)]
        fn collection_id(&self) -> Id {
            self.data.collection_id(self.env().account_id())
        }

        #[ink(message)]
        fn total_supply(&self) -> u128 {
            self.data.total_supply()
        }

        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> u32 {
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

    // (6)
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

    // (7)
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

    // (8)
    impl PSP34Metadata for Token {
        #[ink(message)]
        fn get_attribute(&self, id: Id, key: Vec<u8>) -> Option<Vec<u8>> {
            self.metadata.get_attribute(id, key)
        }
    }

    // (9)
    #[cfg(test)]
    mod tests {
        crate::tests!(Token, Token::new);
    }
}
