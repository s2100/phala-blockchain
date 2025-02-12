#![cfg(test)]

use super::mock::{
	assert_events, balances, expect_event, new_test_ext, Balances, Bridge, BridgeTransfer, Call,
	Event, NativeTokenResourceId, Origin, ProposalLifetime, Test, ENDOWED_BALANCE, RELAYER_A,
	RELAYER_B, RELAYER_C,
};
use super::{bridge, *};
use frame_support::{assert_noop, assert_ok};

use hex_literal::hex;

const TEST_THRESHOLD: u32 = 2;

fn make_transfer_proposal(to: u64, amount: u64) -> Call {
	let resource_id = NativeTokenResourceId::get();
	Call::BridgeTransfer(crate::Call::transfer {
		to,
		amount: amount.into(),
		rid: resource_id,
	})
}

#[test]
fn constant_equality() {
	let r_id = bridge::derive_resource_id(1, &bridge::hashing::blake2_128(b"PHA"));
	let encoded: [u8; 32] =
		hex!("00000000000000000000000000000063a7e2be78898ba83824b0c0cc8dfb6001");
	assert_eq!(r_id, encoded);
}

#[test]
fn do_asset_deposit() {
	new_test_ext().execute_with(|| {
		let asset = bridge::derive_resource_id(2, &bridge::hashing::blake2_128(b"an asset"));
		let amount: u64 = 100;

		// set some balance for holding account and more than amount here
		BridgeBalances::<Test>::insert(asset, Bridge::account_id(), amount * 2);

		BridgeTransfer::do_asset_deposit(&asset, &RELAYER_A, amount);

		assert_eq!(BridgeTransfer::asset_balance(&asset, &RELAYER_A), amount);
		assert_eq!(
			BridgeTransfer::asset_balance(&asset, &Bridge::account_id()),
			amount
		);
	})
}

#[test]
fn do_asset_withdraw() {
	new_test_ext().execute_with(|| {
		let asset = bridge::derive_resource_id(2, &bridge::hashing::blake2_128(b"an asset"));
		let amount: u64 = 100;

		// set some balance for sender account and more than amount here
		BridgeBalances::<Test>::insert(asset, &RELAYER_A, amount * 2);

		BridgeTransfer::do_asset_withdraw(&asset, &RELAYER_A, amount);

		assert_eq!(BridgeTransfer::asset_balance(&asset, &RELAYER_A), amount);
		assert_eq!(
			BridgeTransfer::asset_balance(&asset, &Bridge::account_id()),
			amount
		);
	})
}

#[test]
fn register_asset() {
	new_test_ext().execute_with(|| {
		let r_id = bridge::derive_resource_id(2, &bridge::hashing::blake2_128(b"an asset"));

		assert_ok!(BridgeTransfer::register_asset(
			Origin::root(),
			b"an asset".to_vec(),
			2
		));

		assert_eq!(BridgeAssets::<Test>::contains_key(r_id), true);

		assert_noop!(
			BridgeTransfer::register_asset(Origin::root(), b"an asset".to_vec(), 2),
			Error::<Test>::ResourceIdInUse
		);
	})
}

#[test]
fn mint_asset() {
	new_test_ext().execute_with(|| {
		let asset = bridge::derive_resource_id(2, &bridge::hashing::blake2_128(b"an asset"));
		let bridge_id: u64 = Bridge::account_id();

		assert_noop!(
			BridgeTransfer::mint_asset(Origin::root(), asset, 100),
			Error::<Test>::AssetNotRegistered
		);
		assert_ok!(BridgeTransfer::register_asset(
			Origin::root(),
			b"an asset".to_vec(),
			2
		));
		assert_ok!(BridgeTransfer::mint_asset(Origin::root(), asset, 100));
		assert_eq!(BridgeTransfer::asset_balance(&asset, &bridge_id), 100);
	})
}

#[test]
fn burn_asset() {
	new_test_ext().execute_with(|| {
		let asset = bridge::derive_resource_id(2, &bridge::hashing::blake2_128(b"an asset"));

		assert_noop!(
			BridgeTransfer::burn_asset(Origin::root(), asset, 100),
			Error::<Test>::AssetNotRegistered
		);
		assert_ok!(BridgeTransfer::register_asset(
			Origin::root(),
			b"an asset".to_vec(),
			2
		));
		assert_noop!(
			BridgeTransfer::burn_asset(Origin::root(), asset, 100),
			Error::<Test>::InsufficientBalance
		);
		assert_ok!(BridgeTransfer::mint_asset(Origin::root(), asset, 100));
		assert_eq!(
			BridgeTransfer::asset_balance(&asset, &Bridge::account_id()),
			100
		);
		assert_ok!(BridgeTransfer::burn_asset(Origin::root(), asset, 100));
		assert_eq!(
			BridgeTransfer::asset_balance(&asset, &Bridge::account_id()),
			0
		);
	})
}

#[test]
fn transfer_assets_not_registered() {
	new_test_ext().execute_with(|| {
		let dest_chain = 2;
		let asset =
			bridge::derive_resource_id(dest_chain, &bridge::hashing::blake2_128(b"an asset"));
		let amount: u64 = 100;
		let recipient = vec![99];

		assert_ok!(Bridge::whitelist_chain(Origin::root(), dest_chain.clone()));
		assert_ok!(BridgeTransfer::change_fee(
			Origin::root(),
			2,
			2,
			dest_chain.clone()
		));

		assert_noop!(
			BridgeTransfer::transfer_assets(
				Origin::signed(RELAYER_A),
				asset,
				amount,
				recipient.clone(),
				dest_chain,
			),
			Error::<Test>::AssetNotRegistered
		);
	})
}

#[test]
fn transfer_assets_account_not_exist() {
	new_test_ext().execute_with(|| {
		let dest_chain = 2;
		let asset =
			bridge::derive_resource_id(dest_chain, &bridge::hashing::blake2_128(b"an asset"));
		let amount: u64 = 100;
		let recipient = vec![99];

		assert_ok!(Bridge::whitelist_chain(Origin::root(), dest_chain.clone()));
		assert_ok!(BridgeTransfer::change_fee(
			Origin::root(),
			2,
			2,
			dest_chain.clone()
		));

		assert_ok!(BridgeTransfer::register_asset(
			Origin::root(),
			b"an asset".to_vec(),
			2
		));

		assert_noop!(
			BridgeTransfer::transfer_assets(
				Origin::signed(RELAYER_A),
				asset,
				amount,
				recipient.clone(),
				dest_chain,
			),
			Error::<Test>::AccountNotExist
		);
	})
}

#[test]
fn transfer_assets_insufficient_balance() {
	new_test_ext().execute_with(|| {
		let dest_chain = 2;
		let asset =
			bridge::derive_resource_id(dest_chain, &bridge::hashing::blake2_128(b"an asset"));
		let amount: u64 = 100;
		let recipient = vec![99];

		assert_ok!(Bridge::whitelist_chain(Origin::root(), dest_chain.clone()));
		assert_ok!(BridgeTransfer::change_fee(
			Origin::root(),
			2,
			2,
			dest_chain.clone()
		));

		assert_ok!(BridgeTransfer::register_asset(
			Origin::root(),
			b"an asset".to_vec(),
			2
		));

		// set some balance for account and less than amount here
		BridgeBalances::<Test>::insert(asset, RELAYER_A, amount / 2);

		assert_noop!(
			BridgeTransfer::transfer_assets(
				Origin::signed(RELAYER_A),
				asset,
				amount,
				recipient.clone(),
				dest_chain,
			),
			Error::<Test>::InsufficientBalance
		);
	})
}

#[test]
fn transfer_assets() {
	new_test_ext().execute_with(|| {
		let dest_chain = 2;
		let asset =
			bridge::derive_resource_id(dest_chain, &bridge::hashing::blake2_128(b"an asset"));
		let amount: u64 = 100;
		let recipient = vec![99];

		assert_ok!(Bridge::whitelist_chain(Origin::root(), dest_chain.clone()));
		assert_ok!(BridgeTransfer::change_fee(
			Origin::root(),
			2,
			2,
			dest_chain.clone()
		));

		assert_ok!(BridgeTransfer::register_asset(
			Origin::root(),
			b"an asset".to_vec(),
			2
		));

		// set some balance for account and more than amount here
		BridgeBalances::<Test>::insert(asset, RELAYER_A, amount * 2);

		assert_ok!(BridgeTransfer::transfer_assets(
			Origin::signed(RELAYER_A),
			asset,
			amount,
			recipient.clone(),
			dest_chain,
		));

		assert_eq!(BridgeTransfer::asset_balance(&asset, &RELAYER_A), amount);
		assert_eq!(
			BridgeTransfer::asset_balance(&asset, &Bridge::account_id()),
			amount
		);
	})
}

#[test]
fn transfer_native() {
	new_test_ext().execute_with(|| {
		let dest_chain = 0;
		let resource_id = NativeTokenResourceId::get();
		let amount: u64 = 100;
		let recipient = vec![99];

		assert_ok!(Bridge::whitelist_chain(Origin::root(), dest_chain.clone()));
		assert_ok!(BridgeTransfer::change_fee(
			Origin::root(),
			2,
			2,
			dest_chain.clone()
		));

		assert_noop!(
			BridgeTransfer::transfer_native(
				Origin::signed(RELAYER_A),
				Balances::free_balance(RELAYER_A),
				recipient.clone(),
				dest_chain,
			),
			Error::<Test>::InsufficientBalance
		);

		assert_ok!(BridgeTransfer::transfer_native(
			Origin::signed(RELAYER_A),
			amount.clone(),
			recipient.clone(),
			dest_chain,
		));

		expect_event(bridge::Event::FungibleTransfer(
			dest_chain,
			1,
			resource_id,
			amount.into(),
			recipient,
		));
	})
}

#[test]
fn transfer() {
	new_test_ext().execute_with(|| {
		// Check inital state
		let bridge_id: u64 = Bridge::account_id();
		let resource_id = NativeTokenResourceId::get();
		assert_eq!(Balances::free_balance(&bridge_id), ENDOWED_BALANCE);
		// Transfer and check result
		assert_ok!(BridgeTransfer::transfer(
			Origin::signed(Bridge::account_id()),
			RELAYER_A,
			10,
			resource_id,
		));
		assert_eq!(Balances::free_balance(&bridge_id), ENDOWED_BALANCE - 10);
		assert_eq!(Balances::free_balance(RELAYER_A), ENDOWED_BALANCE + 10);

		assert_events(vec![Event::Balances(balances::Event::Transfer {
			from: Bridge::account_id(),
			to: RELAYER_A,
			amount: 10,
		})]);
	})
}

#[test]
fn transfer_to_holdingaccount() {
	new_test_ext().execute_with(|| {
		let dest_chain = 0;
		let bridge_id: u64 = Bridge::account_id();
		let asset =
			bridge::derive_resource_id(dest_chain, &bridge::hashing::blake2_128(b"an asset"));
		let amount: u64 = 100;

		// transfer to bridge account is not allowed
		assert_noop!(
			BridgeTransfer::transfer(
				Origin::signed(Bridge::account_id()),
				bridge_id,
				amount,
				asset,
			),
			Error::<Test>::InvalidCommand
		);
	})
}

#[test]
fn transfer_to_regular_account() {
	new_test_ext().execute_with(|| {
		let dest_chain = 0;
		let bridge_id: u64 = Bridge::account_id();
		let asset =
			bridge::derive_resource_id(dest_chain, &bridge::hashing::blake2_128(b"an asset"));
		let amount: u64 = 100;

		assert_ok!(BridgeTransfer::register_asset(
			Origin::root(),
			b"an asset".to_vec(),
			dest_chain
		));

		assert_noop!(
			BridgeTransfer::transfer(
				Origin::signed(Bridge::account_id()),
				RELAYER_A,
				amount,
				asset,
			),
			Error::<Test>::InsufficientBalance
		);

		// mint some asset to holding account first
		assert_ok!(BridgeTransfer::mint_asset(
			Origin::root(),
			asset,
			amount * 2
		));

		// transfer to regular account, would withdraw from holding account then deposit to
		// the regular account
		assert_ok!(BridgeTransfer::transfer(
			Origin::signed(Bridge::account_id()),
			RELAYER_A,
			amount,
			asset,
		));

		assert_eq!(BridgeTransfer::asset_balance(&asset, &bridge_id), amount);
		assert_eq!(BridgeTransfer::asset_balance(&asset, &RELAYER_A), amount);
	})
}

#[test]
fn create_successful_transfer_proposal() {
	new_test_ext().execute_with(|| {
		let prop_id = 1;
		let src_id = 1;
		let r_id = bridge::derive_resource_id(src_id, b"transfer");
		let resource = b"BridgeTransfer.transfer".to_vec();
		let proposal = make_transfer_proposal(RELAYER_A, 10);

		assert_ok!(Bridge::set_threshold(Origin::root(), TEST_THRESHOLD,));
		assert_ok!(Bridge::add_relayer(Origin::root(), RELAYER_A));
		assert_ok!(Bridge::add_relayer(Origin::root(), RELAYER_B));
		assert_ok!(Bridge::add_relayer(Origin::root(), RELAYER_C));
		assert_ok!(Bridge::whitelist_chain(Origin::root(), src_id));
		assert_ok!(Bridge::set_resource(Origin::root(), r_id, resource));

		// Create proposal (& vote)
		assert_ok!(Bridge::acknowledge_proposal(
			Origin::signed(RELAYER_A),
			prop_id,
			src_id,
			r_id,
			Box::new(proposal.clone())
		));
		let prop = Bridge::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
		let expected = bridge::ProposalVotes {
			votes_for: vec![RELAYER_A],
			votes_against: vec![],
			status: bridge::ProposalStatus::Initiated,
			expiry: ProposalLifetime::get() + 1,
		};
		assert_eq!(prop, expected);

		// Second relayer votes against
		assert_ok!(Bridge::reject_proposal(
			Origin::signed(RELAYER_B),
			prop_id,
			src_id,
			r_id,
			Box::new(proposal.clone())
		));
		let prop = Bridge::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
		let expected = bridge::ProposalVotes {
			votes_for: vec![RELAYER_A],
			votes_against: vec![RELAYER_B],
			status: bridge::ProposalStatus::Initiated,
			expiry: ProposalLifetime::get() + 1,
		};
		assert_eq!(prop, expected);

		// Third relayer votes in favour
		assert_ok!(Bridge::acknowledge_proposal(
			Origin::signed(RELAYER_C),
			prop_id,
			src_id,
			r_id,
			Box::new(proposal.clone())
		));
		let prop = Bridge::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
		let expected = bridge::ProposalVotes {
			votes_for: vec![RELAYER_A, RELAYER_C],
			votes_against: vec![RELAYER_B],
			status: bridge::ProposalStatus::Approved,
			expiry: ProposalLifetime::get() + 1,
		};
		assert_eq!(prop, expected);

		assert_eq!(Balances::free_balance(RELAYER_A), ENDOWED_BALANCE + 10);
		assert_eq!(
			Balances::free_balance(Bridge::account_id()),
			ENDOWED_BALANCE - 10
		);

		assert_events(vec![
			Event::Bridge(bridge::Event::VoteFor(src_id, prop_id, RELAYER_A)),
			Event::Bridge(bridge::Event::VoteAgainst(src_id, prop_id, RELAYER_B)),
			Event::Bridge(bridge::Event::VoteFor(src_id, prop_id, RELAYER_C)),
			Event::Bridge(bridge::Event::ProposalApproved(src_id, prop_id)),
			Event::Balances(balances::Event::Transfer {
				from: Bridge::account_id(),
				to: RELAYER_A,
				amount: 10,
			}),
			Event::Bridge(bridge::Event::ProposalSucceeded(src_id, prop_id)),
		]);
	})
}
