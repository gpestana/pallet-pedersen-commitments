use super::*;
use crate::{self as pedersen_commitments};
use frame_support::{
    traits::ConstU32,
    weights::{constants, Weight},
	parameter_types,
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic =
	sp_runtime::generic::UncheckedExtrinsic<AccountId, RuntimeCall, (), ()>;

frame_support::construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Event<T>, Config<T>},
		PedersenCommitments: pedersen_commitments::{Pallet, Config, Storage},
	}
);

pub(crate) type Balance = u64;
pub(crate) type AccountId = u64;
pub(crate) type BlockNumber = u64;

impl crate::Config for Runtime {
	type SS58Prefix = ();
	type BaseCallFilter = frame_support::traits::Everything;
	type RuntimeOrigin = RuntimeOrigin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ();
	type DbWeight = ();
	type BlockLength = ();
	type BlockWeights = BlockWeights;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
		::with_sensible_defaults(
			Weight::from_components(2u64 * constants::WEIGHT_PER_SECOND.ref_time(), u64::MAX),
			NORMAL_DISPATCH_RATIO,
		);
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

#[derive(Default)]
pub struct ExtBuilder {}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
		sp_tracing::try_init_simple();
		let mut storage =
			frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

		let _ = pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![
				// bunch of account for submitting stuff only.
				(99, 100),
				(999, 100),
				(9999, 100),
			],
		}
		.assimilate_storage(&mut storage);

		sp_io::TestExternalities::from(storage)
	}

    pub fn build_and_execute(self, test: impl FnOnce() -> ()) {
		self.build().execute_with(test)
	}
}
