#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod data;
mod errors;
mod traits;

pub use data::{Id, PSP34Data, PSP34Event};
pub use errors::PSP34Error;
pub use traits::{PSP34Metadata, PSP34};

#[ink::contract]
mod token {
    use crate::{Id, PSP34Data, PSP34Error, PSP34Event, PSP34};

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
            Ok(self.emit_events(events))
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
            Ok(self.emit_events(events))
        }

        #[ink(message)]
        fn owner_of(&self, id: Id) -> Option<AccountId> {
            self.data.owner_of(&id)
        }

        #[ink(message)]
        fn mint(&mut self, id: Id) -> Result<(), PSP34Error> {
            let events = self.data.mint(self.env().caller(), id)?;
            Ok(self.emit_events(events))
        }

        #[ink(message)]
        fn burn(&mut self, account: AccountId, id: Id) -> Result<(), PSP34Error> {
            let events = self.data.burn(self.env().caller(), account, id)?;
            Ok(self.emit_events(events))
        }
    }

    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        #[ink::test]
        fn mint_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            // Create a new contract instance.
            let mut token = Token::new();
            // Token 1 does not exists.
            assert_eq!(token.owner_of(Id::U8(1)), None);
            // Alice does not owns tokens.
            assert_eq!(token.balance_of(accounts.alice), 0);
            // Create token Id 1.
            assert_eq!(token.mint(Id::U8(1)), Ok(()));
            // Alice owns 1 token.
            assert_eq!(token.balance_of(accounts.alice), 1);
        }

        #[ink::test]
        fn mint_existing_should_fail() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            // Create a new contract instance.
            let mut token = Token::new();
            // Create token Id 1.
            assert_eq!(token.mint(Id::U8(1)), Ok(()));
            // The first Transfer event takes place
            assert_eq!(1, ink::env::test::recorded_events().count());
            // Alice owns 1 token.
            assert_eq!(token.balance_of(accounts.alice), 1);
            // Alice owns token Id 1.
            assert_eq!(token.owner_of(Id::U8(1)), Some(accounts.alice));
            // Cannot create  token Id if it exists.
            // Bob cannot own token Id 1.
            assert_eq!(token.mint(Id::U8(1)), Err(PSP34Error::TokenExists));
        }

        #[ink::test]
        fn transfer_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            // Create a new contract instance.
            let mut token = Token::new();
            // Create token Id 1 for Alice
            assert_eq!(token.mint(Id::U8(1)), Ok(()));
            // Alice owns token 1
            assert_eq!(token.balance_of(accounts.alice), 1);
            // Bob does not owns any token
            assert_eq!(token.balance_of(accounts.bob), 0);
            // The first Transfer event takes place
            assert_eq!(1, ink::env::test::recorded_events().count());
            // Alice transfers token 1 to Bob
            assert_eq!(
                token.transfer(accounts.bob, Id::U8(1), ink::prelude::vec![u8::default()]),
                Ok(())
            );
            // The second Transfer event takes place
            assert_eq!(2, ink::env::test::recorded_events().count());
            // Bob owns token 1
            assert_eq!(token.balance_of(accounts.bob), 1);
        }

        #[ink::test]
        fn invalid_transfer_should_fail() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            // Create a new contract instance.
            let mut token = Token::new();
            // Transfer token fails if it does not exists.
            assert_eq!(
                token.transfer(accounts.bob, Id::U8(2), ink::prelude::vec![u8::default()]),
                Err(PSP34Error::TokenNotExists)
            );
            // Token Id 2 does not exists.
            assert_eq!(token.owner_of(Id::U8(2)), None);
            // Create token Id 2.
            assert_eq!(token.mint(Id::U8(2)), Ok(()));
            // Alice owns 1 token.
            assert_eq!(token.balance_of(accounts.alice), 1);
            // Token Id 2 is owned by Alice.
            assert_eq!(token.owner_of(Id::U8(2)), Some(accounts.alice));
            // Set Bob as caller
            set_caller(accounts.bob);
            // Bob cannot transfer not owned tokens.
            assert_eq!(
                token.transfer(accounts.eve, Id::U8(2), ink::prelude::vec![u8::default()]),
                Err(PSP34Error::NotApproved)
            );
        }

        #[ink::test]
        fn approved_transfer_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            // Create a new contract instance.
            let mut token = Token::new();
            // Create token Id 1.
            assert_eq!(token.mint(Id::U8(1)), Ok(()));
            // Token Id 1 is owned by Alice.
            assert_eq!(token.owner_of(Id::U8(1)), Some(accounts.alice));
            // Approve token Id 1 transfer for Bob on behalf of Alice.
            assert_eq!(token.approve(accounts.bob, Some(Id::U8(1)), true), Ok(()));
            // Set Bob as caller
            set_caller(accounts.bob);
            // Bob transfers token Id 1 from Alice to Eve.
            assert_eq!(
                token.transfer(accounts.eve, Id::U8(1), ink::prelude::vec![u8::default()]),
                Ok(())
            );
            // TokenId 3 is owned by Eve.
            assert_eq!(token.owner_of(Id::U8(1)), Some(accounts.eve));
            // Alice does not owns tokens.
            assert_eq!(token.balance_of(accounts.alice), 0);
            // Bob does not owns tokens.
            assert_eq!(token.balance_of(accounts.bob), 0);
            // Eve owns 1 token.
            assert_eq!(token.balance_of(accounts.eve), 1);
        }

        #[ink::test]
        fn approved_for_all_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            // Create a new contract instance.
            let mut token = Token::new();
            // Create token Id 1.
            assert_eq!(token.mint(Id::U8(1)), Ok(()));
            // Create token Id 2.
            assert_eq!(token.mint(Id::U8(2)), Ok(()));
            // Alice owns 2 tokens.
            assert_eq!(token.balance_of(accounts.alice), 2);
            // Approve all tokens transfer for Bob on behalf of Alice.
            assert_eq!(token.approve(accounts.bob, None, true), Ok(()));
            // Bob is an approved operator for Alice
            assert!(token.allowance(accounts.alice, accounts.bob, None));
            // Set Bob as caller
            set_caller(accounts.bob);
            // Bob transfers token Id 1 from Alice to Eve.
            assert_eq!(
                token.transfer(accounts.eve, Id::U8(1), ink::prelude::vec![u8::default()]),
                Ok(())
            );
            // TokenId 1 is owned by Eve.
            assert_eq!(token.owner_of(Id::U8(1)), Some(accounts.eve));
            // Alice owns 1 token.
            assert_eq!(token.balance_of(accounts.alice), 1);
            // Bob transfers token Id 2 from Alice to Eve.
            assert_eq!(
                token.transfer(accounts.eve, Id::U8(2), ink::prelude::vec![u8::default()]),
                Ok(())
            );
            // Bob does not own tokens.
            assert_eq!(token.balance_of(accounts.bob), 0);
            // Eve owns 2 tokens.
            assert_eq!(token.balance_of(accounts.eve), 2);
            // Remove operator approval for Bob on behalf of Alice.
            set_caller(accounts.alice);
            assert_eq!(token.approve(accounts.bob, None, false), Ok(()));
            // Bob is not an approved operator for Alice.
            assert!(!token.allowance(accounts.alice, accounts.bob, None));
        }

        #[ink::test]
        fn approved_for_all_revoke_single_approval_should_fail() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            // Create a new contract instance.
            let mut token = Token::new();
            // Create token Id 1.
            assert_eq!(token.mint(Id::U8(1)), Ok(()));
            // Create token Id 2.
            assert_eq!(token.mint(Id::U8(2)), Ok(()));
            // Alice owns 2 tokens.
            assert_eq!(token.balance_of(accounts.alice), 2);
            // Approve all tokens transfer for Bob on behalf of Alice.
            assert_eq!(token.approve(accounts.bob, None, true), Ok(()));
            // Bob is an approved operator for Alice
            assert!(token.allowance(accounts.alice, accounts.bob, None));
            // Cannot revoke approval for a single token for Bob
            assert_eq!(token.approve(accounts.bob, Some(Id::U8(1)), false),
                Err(PSP34Error::Custom(String::from(
                    "Cannot revoke approval for a single token, when the operator has approval for all tokens.")))
            );
        }

        #[ink::test]
        fn not_approved_transfer_should_fail() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            // Create a new contract instance.
            let mut token = Token::new();
            // Create token Id 1.
            assert_eq!(token.mint(Id::U8(1)), Ok(()));
            // Alice owns 1 token.
            assert_eq!(token.balance_of(accounts.alice), 1);
            // Bob does not owns tokens.
            assert_eq!(token.balance_of(accounts.bob), 0);
            // Eve does not owns tokens.
            assert_eq!(token.balance_of(accounts.eve), 0);
            // Set Eve as caller
            set_caller(accounts.eve);
            // Eve is not an approved operator by Alice.
            assert_eq!(
                token.transfer(accounts.frank, Id::U8(1), ink::prelude::vec![u8::default()]),
                Err(PSP34Error::NotApproved)
            );
            // Alice owns 1 token.
            assert_eq!(token.balance_of(accounts.alice), 1);
            // Bob does not owns tokens.
            assert_eq!(token.balance_of(accounts.bob), 0);
            // Eve does not owns tokens.
            assert_eq!(token.balance_of(accounts.eve), 0);
        }

        #[ink::test]
        fn burn_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            // Create a new contract instance.
            let mut token = Token::new();
            // Create token Id 1 for Alice
            assert_eq!(token.mint(Id::U8(1)), Ok(()));
            // Alice owns 1 token.
            assert_eq!(token.balance_of(accounts.alice), 1);
            // Alice owns token Id 1.
            assert_eq!(token.owner_of(Id::U8(1)), Some(accounts.alice));
            // Destroy token Id 1.
            assert_eq!(token.burn(accounts.alice, Id::U8(1)), Ok(()));
            // Alice does not owns tokens.
            assert_eq!(token.balance_of(accounts.alice), 0);
            // Token Id 1 does not exists
            assert_eq!(token.owner_of(Id::U8(1)), None);
        }

        #[ink::test]
        fn burn_fails_token_not_found() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            // Create a new contract instance.
            let mut token = Token::new();
            // Try burning a non existent token
            assert_eq!(
                token.burn(accounts.alice, Id::U8(1)),
                Err(PSP34Error::TokenNotExists)
            );
        }

        #[ink::test]
        fn burn_fails_not_owner() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            // Create a new contract instance.
            let mut token = Token::new();
            // Create token Id 1 for Alice
            assert_eq!(token.mint(Id::U8(1)), Ok(()));
            // Try burning this token with a different account
            set_caller(accounts.eve);
            assert_eq!(
                token.burn(accounts.alice, Id::U8(1)),
                Err(PSP34Error::NotApproved)
            );
        }

        fn set_caller(sender: AccountId) {
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(sender);
        }
    }
}
