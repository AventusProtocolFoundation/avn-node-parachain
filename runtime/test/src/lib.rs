#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod proxy_config;
pub mod xcm_config;

use codec::{Decode, Encode};
use scale_info::TypeInfo;

use cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, ConstU128, OpaqueMetadata, H160};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, ConvertInto},
	transaction_validity::{TransactionPriority, TransactionSource, TransactionValidity},
	ApplyExtrinsicResult,
};

use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use frame_support::{
	construct_runtime,
	dispatch::DispatchClass,
	parameter_types,
	traits::{
		AsEnsureOriginWithArg, ConstU32, ConstU64, Contains, Currency, Imbalance, OnUnbalanced,
	},
	weights::{constants::WEIGHT_REF_TIME_PER_SECOND, ConstantMultiplier, Weight},
	PalletId, RuntimeDebug,
};
use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureRoot, EnsureSigned,
};
use proxy_config::AvnProxyConfig;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
pub use sp_runtime::{MultiAddress, Perbill, Permill};
use xcm_config::{XcmConfig, XcmOriginToTransactDispatchOrigin};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

// Polkadot imports
use polkadot_runtime_common::{BlockHashCount, SlowAdjustingFeeUpdate};

// XCM Imports
use xcm_executor::XcmExecutor;

use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_session::historical::{self as pallet_session_historical};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;

use pallet_avn::sr25519::AuthorityId as AvnId;

use pallet_avn_proxy::ProvableProxy;
use sp_avn_common::{InnerCallValidator, Proof};

use pallet_parachain_staking;
pub type NegativeImbalance<T> = <pallet_balances::Pallet<T> as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

/// Logic for the staking pot to get all the fees, including tips.
pub struct ToStakingPot<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for ToStakingPot<R>
where
	R: pallet_balances::Config + pallet_parachain_staking::Config,
	<R as frame_system::Config>::AccountId: From<AccountId>,
	<R as frame_system::Config>::AccountId: Into<AccountId>,
	<R as frame_system::Config>::RuntimeEvent: From<pallet_balances::Event<R>>,
{
	fn on_nonzero_unbalanced(amount: NegativeImbalance<R>) {
		let staking_pot_address =
			<pallet_parachain_staking::Pallet<R>>::compute_reward_pot_account_id();
		<pallet_balances::Pallet<R>>::resolve_creating(&staking_pot_address, amount);
	}
}

pub struct DealWithFees<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for DealWithFees<R>
where
	R: pallet_balances::Config + pallet_parachain_staking::Config,
	<R as frame_system::Config>::AccountId: From<AccountId>,
	<R as frame_system::Config>::AccountId: Into<AccountId>,
	<R as frame_system::Config>::RuntimeEvent: From<pallet_balances::Event<R>>,
{
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance<R>>) {
		if let Some(mut fees) = fees_then_tips.next() {
			if let Some(tips) = fees_then_tips.next() {
				tips.merge_into(&mut fees);
			}
			<ToStakingPot<R> as OnUnbalanced<_>>::on_unbalanced(fees);
		}
	}
}

pub use node_primitives::{AccountId, Signature};
use node_primitives::{Balance, BlockNumber, Hash, Index};

use runtime_common::{
	constants::{currency::*, time::*},
	opaque, weights, Address, Header, OperationalFeeMultiplier, TransactionByteFee, WeightToFee,
};
use weights::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight};

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	(pallet_parachain_staking::migration::EnableStaking<Runtime>, RemoveMigrationStatus),
>;

pub struct RemoveMigrationStatus;
impl frame_support::traits::OnRuntimeUpgrade for RemoveMigrationStatus {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		use frame_support::storage::unhashed;

		use frame_support::storage;
		let storage_prefix = storage::storage_prefix(b"Migration", b"");
		let mut key = vec![0u8; 32];
		key[0..32].copy_from_slice(&storage_prefix);
		let res = unhashed::clear_prefix(&key[0..16], None, None);

		log::info!("✅ Cleared '{}' backend values from 'Migration' storage prefix", res.backend);

		log::info!("✅ Cleared '{}' entries from 'Migration' storage prefix", res.unique);

		if res.maybe_cursor.is_some() {
			log::error!("Storage prefix 'Migration' is not completely cleared.");
		}

		<Runtime as frame_system::Config>::DbWeight::get().writes(1)
	}
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
		pub authority_discovery: AuthorityDiscovery,
		pub im_online: ImOnline,
		pub avn: Avn,
	}
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("avn-test-parachain"),
	impl_name: create_runtime_str!("avn-test-parachain"),
	authoring_version: 1,
	spec_version: 32,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

/// The existential deposit. Set to 1/10 of the Connected Relay Chain.
pub const EXISTENTIAL_DEPOSIT: Balance = 0;

/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// We allow for 0.5 of a second of compute with a 12 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
	WEIGHT_REF_TIME_PER_SECOND.saturating_div(2),
	cumulus_primitives_core::relay_chain::v2::MAX_POV_SIZE as u64,
);

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;

	// This part is copied from Substrate's `bin/node/runtime/src/lib.rs`.
	//  The `RuntimeBlockLength` and `RuntimeBlockWeights` exist here because the
	// `DeletionWeightLimit` and `DeletionQueueDepth` depend on those to parameterize
	// the lazy contract deletion.
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u16 = 42;
}

// Configure FRAME pallets to include in runtime.

/// Use this filter to block users from calling extrinsics listed here.
pub struct RestrictedEndpointFilter;
impl Contains<RuntimeCall> for RestrictedEndpointFilter {
	fn contains(c: &RuntimeCall) -> bool {
		!matches!(
			c,
			RuntimeCall::ParachainStaking(pallet_parachain_staking::Call::join_candidates { .. }) |
				RuntimeCall::ParachainStaking(
					pallet_parachain_staking::Call::schedule_leave_candidates { .. }
				) | RuntimeCall::ParachainStaking(
				pallet_parachain_staking::Call::execute_leave_candidates { .. }
			) | RuntimeCall::ParachainStaking(
				pallet_parachain_staking::Call::cancel_leave_candidates { .. }
			) // Allow the following direct staking extrinsics:
			  /*
				  Call::ParachainStaking(pallet_parachain_staking::Call::nominate {..}) |
				  Call::ParachainStaking(pallet_parachain_staking::Call::nominator_bond_more {..}) |
				  Call::ParachainStaking(pallet_parachain_staking::Call::schedule_nominator_bond_less {..}) |
				  Call::ParachainStaking(pallet_parachain_staking::Call::schedule_revoke_nomination {..}) |
				  Call::ParachainStaking(pallet_parachain_staking::Call::execute_nomination_request {..}) |
				  Call::ParachainStaking(pallet_parachain_staking::Call::cancel_nomination_request {..}) |

				  Call::ParachainStaking(pallet_parachain_staking::Call::schedule_candidate_bond_less {..}) |
				  Call::ParachainStaking(pallet_parachain_staking::Call::execute_candidate_bond_less {..}) |
				  Call::ParachainStaking(pallet_parachain_staking::Call::cancel_candidate_bond_less {..}) |

				  Call::ParachainStaking(pallet_parachain_staking::Call::schedule_leave_nominators {..}) |
				  Call::ParachainStaking(pallet_parachain_staking::Call::execute_leave_nominators {..}) |
				  Call::ParachainStaking(pallet_parachain_staking::Call::cancel_leave_nominators {..}) |

				  Call::ParachainStaking(pallet_parachain_staking::Call::hotfix_remove_nomination_requests_exited_candidates{..})
			  */
		)
	}
}

impl frame_system::Config for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	// type Call = Call;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// Runtime version.
	type Version = Version;
	/// Converts a module to an index of this module in the runtime.
	type PalletInfo = PalletInfo;
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = RestrictedEndpointFilter;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	/// The action to take on a Runtime Upgrade
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	type WeightInfo = ();
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type UncleGenerations = ConstU32<0>;
	type FilterUncle = ();
	type EventHandler = (ParachainStaking,);
}

parameter_types! {
	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = ConstU32<50>;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction =
		pallet_transaction_payment::CurrencyAdapter<Balances, DealWithFees<Runtime>>;
	type WeightToFee = WeightToFee;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type OutboundXcmpMessageSource = XcmpQueue;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = ();
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = ();
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
	pub const Period: u32 = 6 * HOURS;
	pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = ConvertInto;
	type ShouldEndSession = ParachainStaking;
	type NextSessionRotation = ParachainStaking;
	type SessionManager = ParachainStaking;
	// Essentially just Aura, but let's be pedantic.
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = ();
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<100_000>;
}

parameter_types! {
	// The accountId that will hold the reward for the staking pallet
	pub const RewardPotId: PalletId = PalletId(*b"av/vamgr");
}
impl pallet_parachain_staking::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	/// Minimum era length is 4 minutes (20 * 12 second block times)
	type MinBlocksPerEra = ConstU32<20>;
	/// Eras before the reward is paid
	type RewardPaymentDelay = ConstU32<2>;
	/// Minimum collators selected per era, default at genesis and minimum forever after
	type MinSelectedCandidates = ConstU32<20>;
	/// Maximum top nominations per candidate
	type MaxTopNominationsPerCandidate = ConstU32<300>;
	/// Maximum bottom nominations per candidate
	type MaxBottomNominationsPerCandidate = ConstU32<50>;
	/// Maximum nominations per nominator
	type MaxNominationsPerNominator = ConstU32<100>;
	/// Minimum stake required to be reserved to be a nominator
	type MinNominationPerCollator = ConstU128<1>;
	type RewardPotId = RewardPotId;
	type ErasPerGrowthPeriod = ConstU32<30>; // 30 eras (~ 1 month if era = 1 day)
	type ProcessedEventsChecker = EthereumEvents;
	type Public = <Signature as sp_runtime::traits::Verify>::Signer;
	type Signature = Signature;
	type CollatorSessionRegistration = Session;
	type CollatorPayoutDustHandler = TokenManager;
	type WeightInfo = pallet_parachain_staking::weights::SubstrateWeight<Runtime>;
}

// Substrate pallets that AvN has dependency
impl pallet_authority_discovery::Config for Runtime {
	type MaxAuthorities = ConstU32<100_000>;
}

impl pallet_session::historical::Config for Runtime {
	type FullIdentification = AccountId;
	type FullIdentificationOf = ConvertInto;
}

impl pallet_offences::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
	type OnOffenceHandler = AvnOffenceHandler;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	RuntimeCall: From<C>,
{
	type Extrinsic = UncheckedExtrinsic;
	type OverarchingCall = RuntimeCall;
}

parameter_types! {
	pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
	pub const MaxKeys: u16 = 100;
	pub const MaxPeerInHeartbeats: u32 = 10_000;
	pub const MaxPeerDataEncodingSize: u32 = 1_000;
}

impl pallet_im_online::Config for Runtime {
	type AuthorityId = ImOnlineId;
	type RuntimeEvent = RuntimeEvent;
	type NextSessionRotation = ParachainStaking;
	type ValidatorSet = Historical;
	type ReportUnresponsiveness = Offences;
	type UnsignedPriority = ImOnlineUnsignedPriority;
	type WeightInfo = pallet_im_online::weights::SubstrateWeight<Runtime>;
	type MaxKeys = MaxKeys;
	type MaxPeerInHeartbeats = MaxPeerInHeartbeats;
	type MaxPeerDataEncodingSize = MaxPeerDataEncodingSize;
}

impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

// AvN pallets
impl pallet_avn_offence_handler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Enforcer = ValidatorsManager;
	type WeightInfo = pallet_avn_offence_handler::default_weights::SubstrateWeight<Runtime>;
}

impl pallet_avn::Config for Runtime {
	type AuthorityId = AvnId;
	type EthereumPublicKeyChecker = ValidatorsManager;
	type NewSessionHandler = ValidatorsManager;
	type DisabledValidatorChecker = ValidatorsManager;
	type FinalisedBlockChecker = AvnFinalityTracker;
}

parameter_types! {
	pub const MinEthBlockConfirmation: u64 = 20;
}

impl pallet_ethereum_events::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type ProcessedEventHandler = (TokenManager, NftManager);
	type MinEthBlockConfirmation = MinEthBlockConfirmation;
	type Public = <Signature as sp_runtime::traits::Verify>::Signer;
	type Signature = Signature;
	type ReportInvalidEthereumLog = Offences;
	type WeightInfo = pallet_ethereum_events::default_weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub ValidatorManagerContractAddress: H160 = pallet_ethereum_events::ValidatorManagerContractAddress::<Runtime>::get();
}

impl pallet_ethereum_transactions::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type AccountToBytesConvert = Avn;
	type ValidatorManagerContractAddress = ValidatorManagerContractAddress;
	type WeightInfo = pallet_ethereum_transactions::default_weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const ValidatorManagerVotingPeriod: BlockNumber = 30 * MINUTES;
}

impl pallet_validators_manager::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ProcessedEventsChecker = EthereumEvents;
	type VotingPeriod = ValidatorManagerVotingPeriod;
	type CandidateTransactionSubmitter = EthereumTransactions;
	type AccountToBytesConvert = Avn;
	type ValidatorRegistrationNotifier = AvnOffenceHandler;
	type ReportValidatorOffence = Offences;
	type WeightInfo = pallet_validators_manager::default_weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const CacheAge: BlockNumber = 10;
	pub const SubmissionInterval: BlockNumber = 5;
	pub const MaxAllowedReportLatency: BlockNumber = 5 * MINUTES;
}

impl pallet_avn_finality_tracker::pallet::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CacheAge = CacheAge;
	type SubmissionInterval = SubmissionInterval;
	type ReportLatency = MaxAllowedReportLatency;
	type WeightInfo = pallet_avn_finality_tracker::default_weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const AdvanceSlotGracePeriod: BlockNumber = 5;
	pub const MinBlockAge: BlockNumber = 5;
	pub const AvnTreasuryPotId: PalletId = PalletId(*b"Treasury");
	pub const TreasuryGrowthPercentage: Perbill = Perbill::from_percent(75);
}

impl pallet_summary::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AdvanceSlotGracePeriod = AdvanceSlotGracePeriod;
	type MinBlockAge = MinBlockAge;
	type CandidateTransactionSubmitter = EthereumTransactions;
	type AccountToBytesConvert = Avn;
	type ReportSummaryOffence = Offences;
	type FinalityReportLatency = MaxAllowedReportLatency;
	type WeightInfo = pallet_summary::default_weights::SubstrateWeight<Runtime>;
}

pub type EthAddress = H160;

impl pallet_token_manager::pallet::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type TokenBalance = Balance;
	type TokenId = EthAddress;
	type ProcessedEventsChecker = EthereumEvents;
	type Public = <Signature as sp_runtime::traits::Verify>::Signer;
	type Signature = Signature;
	type OnGrowthLiftedHandler = ParachainStaking;
	type TreasuryGrowthPercentage = TreasuryGrowthPercentage;
	type AvnTreasuryPotId = AvnTreasuryPotId;
	type WeightInfo = pallet_token_manager::default_weights::SubstrateWeight<Runtime>;
}

impl pallet_nft_manager::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type ProcessedEventsChecker = EthereumEvents;
	type Public = <Signature as sp_runtime::traits::Verify>::Signer;
	type Signature = Signature;
	type WeightInfo = pallet_nft_manager::default_weights::SubstrateWeight<Runtime>;
}

impl pallet_avn_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type Public = <Signature as sp_runtime::traits::Verify>::Signer;
	type Signature = Signature;
	type ProxyConfig = AvnProxyConfig;
	type WeightInfo = pallet_avn_proxy::default_weights::SubstrateWeight<Runtime>;
}

// Other pallets
parameter_types! {
	pub const AssetDeposit: Balance = 10 * MILLI_AVT;
	pub const ApprovalDeposit: Balance = 100 * MICRO_AVT;
	pub const StringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 1 * MILLI_AVT;
	pub const MetadataDepositPerByte: Balance = 100 * MICRO_AVT;
}
const ASSET_ACCOUNT_DEPOSIT: Balance = 100 * MICRO_AVT;

impl pallet_assets::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = u128;
	type RemoveItemsLimit = ConstU32<5>;
	type AssetId = u32;
	type AssetIdParameter = u32;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = ConstU128<ASSET_ACCOUNT_DEPOSIT>;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		// System support stuff.
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
		ParachainSystem: cumulus_pallet_parachain_system::{
			Pallet, Call, Config, Storage, Inherent, Event<T>, ValidateUnsigned,
		} = 1,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 2,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 3,

		// Monetary stuff.
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 10,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>} = 11,

		// Collator support. The order of these 4 are important and shall not change.
		Authorship: pallet_authorship::{Pallet, Call, Storage} = 20,
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 22,
		Aura: pallet_aura::{Pallet, Storage, Config<T>} = 23,
		AuraExt: cumulus_pallet_aura_ext::{Pallet, Storage, Config} = 24,
		ParachainStaking: pallet_parachain_staking::{Pallet, Call, Storage, Event<T>, Config<T>} = 96,

		// Since the ValidatorsManager integrates with the ParachainStaking pallet, we want to initialise after it.
		ValidatorsManager: pallet_validators_manager = 18,

		// XCM helpers.
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 30,
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin, Config} = 31,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 32,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 33,

		// Substrate pallets
		Assets: pallet_assets = 60,
		Sudo: pallet_sudo = 62,
		AuthorityDiscovery: pallet_authority_discovery = 70,
		Historical: pallet_session_historical::{Pallet} = 71,
		Offences: pallet_offences = 72,
		ImOnline: pallet_im_online = 73,
		Utility: pallet_utility = 74,

		// Rest of AvN pallets
		Avn: pallet_avn = 81,
		AvnFinalityTracker: pallet_avn_finality_tracker = 82,
		AvnOffenceHandler: pallet_avn_offence_handler = 83,
		EthereumEvents: pallet_ethereum_events = 84,
		EthereumTransactions: pallet_ethereum_transactions = 85,
		NftManager: pallet_nft_manager = 86,
		TokenManager: pallet_token_manager = 87,
		Summary: pallet_summary = 88,
		AvnProxy: pallet_avn_proxy = 89,
	}
);

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!(
		[frame_system, SystemBench::<Runtime>]
		[pallet_assets, Assets]
		[pallet_balances, Balances]
		[pallet_avn_finality_tracker, AvnFinalityTracker]
		[pallet_avn_offence_handler, AvnOffenceHandler]
		[pallet_avn_proxy, AvnProxy]
		[pallet_ethereum_events, EthereumEvents]
		[pallet_ethereum_transactions, EthereumTransactions]
		[pallet_nft_manager, NftManager]
		[pallet_summary, Summary]
		[pallet_token_manager, TokenManager]
		[pallet_validators_manager, ValidatorsManager]
		[pallet_session, SessionBench::<Runtime>]
		[pallet_timestamp, Timestamp]
		[pallet_utility, Utility]
		[pallet_parachain_staking, ParachainStaking]
		[cumulus_pallet_xcmp_queue, XcmpQueue]
	);
}

impl_runtime_apis! {
	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
		for Runtime
	{
		fn query_call_info(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
	}
	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: bool) -> (Weight, Weight) {
			log::info!("try-runtime::on_runtime_upgrade avn-parachain.");
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, RuntimeBlockWeights::get().max_block)
		}

		fn execute_block(
			block: Block,
			state_root_check: bool,
			signature_check: bool,
			select: frame_try_runtime::TryStateSelect,
		) -> Weight {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here.
			Executive::try_execute_block(block, state_root_check, signature_check, select).unwrap()
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();
			return (list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, TrackedStorageKey};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			impl cumulus_pallet_session_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}

	impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
		fn authorities() -> Vec<AuthorityDiscoveryId> {
			AuthorityDiscovery::authorities()
		}
	}
}

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data =
			cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
				relay_chain_slot,
				sp_std::time::Duration::from_secs(6),
			)
			.create_inherent_data()
			.expect("Could not create the timestamp inherent data");

		inherent_data.check_extrinsics(block)
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
}
