/// Inserts a suite of ink! unit tests intended for a contract implementing PSP34 trait.
/// `$contract` argument should be the name of the contract struct.
/// `$constructor` argument should be the name of a function, which initializes `$contract`.
/// This macro should be invoked inside `#[ink::contract]` module.
#[macro_export]
macro_rules! tests {
    ($contract:ident, $constructor:expr) => {
        mod psp34_unit_tests {
            use super::super::*;
            use ink::env::{test::*, DefaultEnvironment as E};

            type Event = <$contract as ::ink::reflect::ContractEventBase>::Type;

            // Gathers all emitted events, skip `shift` first, decode the rest and return as vector
            fn decode_events(shift: usize) -> Vec<Event> {
                recorded_events()
                    .skip(shift)
                    .map(|e| <Event as scale::Decode>::decode(&mut &e.data[..]).unwrap())
                    .collect()
            }

            // Asserts if the given event is a Transfer with particular from_, to_ and value_
            fn assert_transfer(event: &Event, from_: AccountId, to_: AccountId, id_: Id) {
                if let Event::Transfer(Transfer { from, to, id }) = event {
                    assert_eq!(*from, Some(from_), "Transfer event: 'from' mismatch");
                    assert_eq!(*to, Some(to_), "Transfer event: 'to' mismatch");
                    assert_eq!(*id, id_, "Transfer event: 'id' mismatch");
                } else {
                    panic!("Event is not Transfer")
                }
            }

            // Asserts if the given event is a Approval with particular owner_, spender_ and amount_
            fn assert_approval(
                event: &Event,
                owner_: AccountId,
                operator_: AccountId,
                id_: Option<Id>,
                approved_ : bool,
            ) {
                if let Event::Approval(Approval {
                    owner,
                    operator,
                    id,
                    approved,
                }) = event
                {
                    assert_eq!(*owner, owner_, "Approval event: 'owner' mismatch");
                    assert_eq!(*operator, operator_, "Approval event: 'operator' mismatch");
                    assert_eq!(*id, id_, "Approval event: 'id' mismatch");
                    assert_eq!(*approved, approved_, "Approval event: 'approved' mismatch")
                } else {
                    panic!("Event is not Approval")
                }
            }
            
            fn set_caller(sender: AccountId) {
                ink::env::test::set_caller::<E>(sender);
            }

            #[ink::test]
            fn mint_works() {
                let accounts = default_accounts::<E>();
                // Create a new contract instance.
                let mut token = $constructor();
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
                let accounts = default_accounts::<E>();
                // Create a new contract instance.
                let mut token = $constructor();
                // Create token Id 1.
                assert_eq!(token.mint(Id::U8(1)), Ok(()));
                // The first Transfer event takes place
                assert_eq!(1, recorded_events().count());
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
                let accounts = default_accounts::<E>();
                // Create a new contract instance.
                let mut token = $constructor();
                // Create token Id 1 for Alice
                assert_eq!(token.mint(Id::U8(1)), Ok(()));
                // Alice owns token 1
                assert_eq!(token.balance_of(accounts.alice), 1);
                // Bob does not owns any token
                assert_eq!(token.balance_of(accounts.bob), 0);
                // The first Transfer event takes place
                assert_eq!(1, recorded_events().count());
                // Alice transfers token 1 to Bob
                assert_eq!(
                    token.transfer(accounts.bob, Id::U8(1), vec![u8::default()]),
                    Ok(())
                );
                // The second Transfer event takes place
                assert_eq!(2, recorded_events().count());
                // Bob owns token 1
                assert_eq!(token.balance_of(accounts.bob), 1);
            }

            #[ink::test]
            fn transfer_emits_event() {                
                let accounts = default_accounts::<E>();
                let start = recorded_events().count();
                // Create a new contract instance.
                let mut token = $constructor();
                // Create token Id 1 for Alice
                assert_eq!(token.mint(Id::U8(1)), Ok(()));
                // Alice owns token 1
                assert_eq!(token.balance_of(accounts.alice), 1);
                // Bob does not owns any token
                assert_eq!(token.balance_of(accounts.bob), 0);
                // The first Transfer event takes place
                assert_eq!(1, recorded_events().count());
                // Alice transfers token 1 to Bob
                assert_eq!(
                    token.transfer(accounts.bob, Id::U8(1), vec![u8::default()]),
                    Ok(())
                );                
                // The second Transfer event takes place
                assert_eq!(2, recorded_events().count());
                // The correct event emited
                let events = decode_events(start);
                assert_transfer(&events[1], accounts.alice, accounts.bob, Id::U8(1));
            }

            #[ink::test]
            fn invalid_transfer_should_fail() {
                let accounts = default_accounts::<E>();
                // Create a new contract instance.
                let mut token = $constructor();
                // Transfer token fails if it does not exists.
                assert_eq!(
                    token.transfer(accounts.bob, Id::U8(2), vec![u8::default()]),
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
                    token.transfer(accounts.eve, Id::U8(2), vec![u8::default()]),
                    Err(PSP34Error::NotApproved)
                );
            }

            #[ink::test]
            fn approved_transfer_works() {
                let accounts = default_accounts::<E>();
                // Create a new contract instance.
                let mut token = $constructor();
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
                    token.transfer(accounts.eve, Id::U8(1), vec![u8::default()]),
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
            fn approve_emits_event() {                
                let accounts = default_accounts::<E>();
                let start = recorded_events().count();
                // Create a new contract instance.
                let mut token = $constructor();
                // Create token Id 1.
                assert_eq!(token.mint(Id::U8(1)), Ok(()));
                // Token Id 1 is owned by Alice.
                assert_eq!(token.owner_of(Id::U8(1)), Some(accounts.alice));
                // Approve token Id 1 transfer for Bob on behalf of Alice.
                assert_eq!(token.approve(accounts.bob, Some(Id::U8(1)), true), Ok(()));
                // The event approve event takes place
                let events = decode_events(start);
                assert_eq!(events.len(), 2);
                assert_approval(&events[1], accounts.alice, accounts.bob, Some(Id::U8(1)), true);
            }

            #[ink::test]
            fn approved_for_all_works() {
                let accounts = default_accounts::<E>();
                // Create a new contract instance.
                let mut token = $constructor();
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
                    token.transfer(accounts.eve, Id::U8(1), vec![u8::default()]),
                    Ok(())
                );
                // TokenId 1 is owned by Eve.
                assert_eq!(token.owner_of(Id::U8(1)), Some(accounts.eve));
                // Alice owns 1 token.
                assert_eq!(token.balance_of(accounts.alice), 1);
                // Bob transfers token Id 2 from Alice to Eve.
                assert_eq!(
                    token.transfer(accounts.eve, Id::U8(2), vec![u8::default()]),
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
                let accounts = default_accounts::<E>();
                // Create a new contract instance.
                let mut token = $constructor();
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
                let accounts = default_accounts::<E>();
                // Create a new contract instance.
                let mut token = $constructor();
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
                    token.transfer(accounts.frank, Id::U8(1), vec![u8::default()]),
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
                let accounts = default_accounts::<E>();
                // Create a new contract instance.
                let mut token = $constructor();
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
                let accounts = default_accounts::<E>();
                // Create a new contract instance.
                let mut token = $constructor();
                // Try burning a non existent token
                assert_eq!(
                    token.burn(accounts.alice, Id::U8(1)),
                    Err(PSP34Error::TokenNotExists)
                );
            }

            #[ink::test]
            fn burn_fails_not_owner() {
                let accounts = default_accounts::<E>();
                // Create a new contract instance.
                let mut token = $constructor();
                // Create token Id 1 for Alice
                assert_eq!(token.mint(Id::U8(1)), Ok(()));
                // Try burning this token with a different account
                set_caller(accounts.eve);
                assert_eq!(
                    token.burn(accounts.alice, Id::U8(1)),
                    Err(PSP34Error::NotApproved)
                );
            }
        }
    };
}
