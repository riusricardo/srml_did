/// Tests for module DID

#[cfg(test)]
mod tests {
    use crate::did;
	use support::{impl_outer_origin, assert_ok, assert_noop};
	use runtime_io::{with_externalities, TestExternalities};
	use primitives::{H256, Blake2Hasher};
	use runtime_primitives::{
		BuildStorage,
        traits::{BlakeTwo256, IdentityLookup},
		testing::{Digest, DigestItem, Header}
	};

	impl_outer_origin! {
		pub enum Origin for DIDTest {}
	}

	#[derive(Clone, Eq, PartialEq)]
	pub struct DIDTest;
	impl system::Trait for DIDTest {
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type Digest = Digest;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type Log = DigestItem;
	}
	
	impl balances::Trait for DIDTest {
		type Balance = u64;
		type OnFreeBalanceZero = ();
		type OnNewAccount = ();
		type Event = ();
		type TransactionPayment = ();
		type TransferPayment = ();
		type DustRemoval = ();
	}

	impl timestamp::Trait for DIDTest {
        type Moment = u64;
        type OnTimestampSet = ();
	}

	impl did::Trait for DIDTest {
		type Event = ();
	}

	fn new_test_ext() -> TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::<DIDTest>::default().build_storage().unwrap().0;
        t.extend(balances::GenesisConfig::<DIDTest>::default().build_storage().unwrap().0);
        TestExternalities::new(t)
	}

    type DID = did::Module<DIDTest>;
    type System = system::Module<DIDTest>;
    type Moment = timestamp::Module<DIDTest>;

	#[test]
	fn transfer_ownership_should_work() {
        with_externalities(&mut new_test_ext(), || {
            
            // Get the owner of an identity
            assert_eq!(DID::identity_owner(&1),1);

            // Verify identity owner
            assert_ok!(DID::is_owner(&1,&1));

            // Transfer identity ownership
            assert_ok!(DID::change_owner(Origin::signed(1), 1, 2));

            // Previous owner is invalid
            assert_noop!(DID::is_owner(&1,&1),"invalid owner");

            // Verify new owner
            assert_ok!(DID::is_owner(&1,&2));

            // Get the new owner of an identity
            assert_eq!(DID::identity_owner(&1),2);
        })
	}

    #[test]
	fn owner_as_delegate_should_work() {
        with_externalities(&mut new_test_ext(), || {

            System::set_block_number(1);

            // Owner is a valid degate for any type and time
            assert_ok!(DID::valid_delegate(Origin::signed(99),1,vec![7,7,7],1));

            System::set_block_number(1000);

            // Owner is a valid degate for any type and time
            assert_ok!(DID::valid_delegate(Origin::signed(99),1,vec![9,9,9],1));

            System::set_block_number(2000);

            // Transfer identity ownership to AccountId-2
            assert_ok!(DID::change_owner(Origin::signed(1), 1, 2));

            // Previous identity owner should be an invalid delegate
            assert_noop!(DID::valid_delegate(Origin::signed(99),1,vec![7,7,7],1),"invalid delegate");

            // New owner is a valid delegate for any type and time
            assert_ok!(DID::valid_delegate(Origin::signed(99),1,vec![8,8,8],2));

        })
	}

    #[test]
	fn add_delegate_should_work() {
        with_externalities(&mut new_test_ext(), || {

            // Should fail to explicity set owner(AccountId-1) in the delegates list
            assert_noop!(DID::add_delegate(Origin::signed(1),1,1,vec![7,7,7],20),"owner cannot be explicity set as delegate");

            // AccountId-5 is an invalid delegate previous to adding it
            assert_noop!(DID::valid_delegate(Origin::signed(99),1,vec![7,7,7],5),"invalid delegate");

            // Add AccountId-5 as delegate of AccountId-1 for a period of 20 blocks
            assert_ok!(DID::add_delegate(Origin::signed(1),1,5,vec![7,7,7],20));

            // AccountId-5 is a valid for a specified type
            assert_ok!(DID::valid_delegate(Origin::signed(99),1,vec![7,7,7],5));

            // AccountId-5 is an invalid delegate for a different type
            assert_noop!(DID::valid_delegate(Origin::signed(99),1,vec![8,8,8],5),"invalid delegate");

        })
	}

    #[test]
	fn delegate_expiration_should_work() {
        with_externalities(&mut new_test_ext(), || {

            System::set_block_number(1);

            // Add AccountId-5 as delegate of AccountId-1 for a period of 3 blocks
            assert_ok!(DID::add_delegate(Origin::signed(1),1,5,vec![7,7,7],3));

            System::set_block_number(3);
            
            // AccountId-5 is a valid specific type delegate
            assert_ok!(DID::valid_delegate(Origin::signed(99),1,vec![7,7,7],5));

            System::set_block_number(4);

            // AccountId-5 is an invalid delegate after expiration
            assert_noop!(DID::valid_delegate(Origin::signed(99),1,vec![7,7,7],5),"invalid delegate");

        })
	}

    #[test]
	fn revoke_delegate_should_work() {
        with_externalities(&mut new_test_ext(), || {

            System::set_block_number(1);

            // Add AccountId-5 as delegate of AccountId-1 for a period of 1000 blocks
            assert_ok!(DID::add_delegate(Origin::signed(1),1,5,vec![7,7,7],1000));

            System::set_block_number(50);
            
            // AccountId-5 is a valid specific type delegate
            assert_ok!(DID::valid_delegate(Origin::signed(99),1,vec![7,7,7],5));

            // AccountId-5 is a revoked delegate from AccountId-1
            assert_ok!(DID::revoke_delegate(Origin::signed(1),1,vec![7,7,7],5));

            // Delegate max valid block is current block
            assert_eq!(DID::delegate_of((1,vec![7,7,7],5)),Some(50));

            System::set_block_number(51);

            // AccountId-5 is an invalid delegate after revocation
            assert_noop!(DID::valid_delegate(Origin::signed(99),1,vec![7,7,7],5),"invalid delegate");

        })
	}

    #[test]
	fn add_attribute_should_work() {
        with_externalities(&mut new_test_ext(), || {

            System::set_block_number(1);
            
            // Add a new attribute to an identity. Valid until block 1 + 1000.
            assert_ok!(DID::add_attribute(Origin::signed(1),1,vec![1,2,3],vec![7,7,7],1000));

            let (attr, _) = DID::attribute_and_id(&1, &vec![1,2,3]).unwrap();
            
            // Validate attribute fields.
            assert_eq!(attr.name, vec![1,2,3]);
            assert_eq!(attr.value, vec![7,7,7]);
            assert_eq!(attr.validity, 1000 + System::block_number());
            assert_eq!(attr.creation, Moment::now());
            assert_eq!(attr.nonce, 0);
            
            System::set_block_number(1000);

            // Validate that the attribute exists and has not expired.
            assert_ok!(DID::valid_attribute(Origin::signed(99),1,vec![1,2,3],vec![7,7,7]));
            
            System::set_block_number(1001);

            // Validate attribute expiration.
            assert_noop!(DID::valid_attribute(Origin::signed(99),1,vec![1,2,3],vec![7,7,7]),"invalid attribute");

        })
	}

    #[test]
	fn revoke_attribute_should_work() {
        with_externalities(&mut new_test_ext(), || {

            System::set_block_number(50);
            
            // Add a new attribute to an identity on block 50 for a validity of 50 + 100 blocks
            // Valid until block 150.
            assert_ok!(DID::add_attribute(Origin::signed(1),1,vec![1,2,3],vec![7,7,7],100));
            
            System::set_block_number(110);

            // Revoke attribute from an identity by setting its expiration to actual block number.
            assert_ok!(DID::revoke_attribute(Origin::signed(1),1,vec![1,2,3]));

            System::set_block_number(111);

            // Attribute should be invalid after revocation. 
            assert_noop!(DID::valid_attribute(Origin::signed(99),1,vec![1,2,3],vec![7,7,7]),"invalid attribute");
        })
	}

    #[test]
	fn delete_attribute_should_work() {
        with_externalities(&mut new_test_ext(), || {

            System::set_block_number(50);

            // Add a new attribute to an identity on block 50 for a validity of 50 + 100 blocks
            // Valid until block 150.
            assert_ok!(DID::add_attribute(Origin::signed(1),1,vec![1,2,3],vec![7,7,7],100));
            
            System::set_block_number(110);

            // Delete attribute from identity on block 110.
            assert_ok!(DID::delete_attribute(Origin::signed(1),1,vec![1,2,3]));

            System::set_block_number(120);

            // Attribute becomes unavailable.
            assert_noop!(DID::valid_attribute(Origin::signed(99),1,vec![1,2,3],vec![7,7,7]),"invalid attribute");
        })
	}
}
