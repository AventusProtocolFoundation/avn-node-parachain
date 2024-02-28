mod local;
mod staging;

pub use local::avn_garde_local_config;
pub use staging::avn_garde_staging_config;

use crate::chain_spec::{
	constants::*, helpers::*, AuraId, AuthorityDiscoveryId, AvnId, EthPublicKey, Extensions,
	ImOnlineId, ParaId,
};
use avn_test_runtime::{
	self as avn_test_runtime, AuthorityDiscoveryConfig, EthereumEventsConfig,
	EthereumTransactionsConfig, ImOnlineConfig, ParachainStakingConfig, SudoConfig, SummaryConfig,
	TokenManagerConfig, ValidatorsManagerConfig,
};
use node_primitives::AccountId;

use sp_core::{H160, H256};

use hex_literal::hex;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<avn_test_runtime::GenesisConfig, Extensions>;

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn avn_session_keys(
	aura_keys: AuraId,
	audi_keys: AuthorityDiscoveryId,
	im_online_keys: ImOnlineId,
	avn_keys: AvnId,
) -> avn_test_runtime::SessionKeys {
	avn_test_runtime::SessionKeys {
		aura: aura_keys,
		authority_discovery: audi_keys,
		im_online: im_online_keys,
		avn: avn_keys,
	}
}

pub(crate) fn avn_test_runtime_genesis(
	candidates: Vec<(AccountId, AuraId, AuthorityDiscoveryId, ImOnlineId, AvnId)>,
	endowed_accounts: Vec<(AccountId, Balance)>,
	id: ParaId,
	sudo_account: AccountId,
	avt_token_contract: H160,
	avn_eth_contract: H160,
	nft_eth_contracts: Vec<(H160, ())>,
	eth_public_keys: Vec<EthPublicKey>,
	lift_tx_hashes: Vec<H256>,
	event_challenge_period: BlockNumber,
	schedule_period: BlockNumber,
	voting_period: BlockNumber,
) -> avn_test_runtime::GenesisConfig {
	avn_test_runtime::GenesisConfig {
		system: avn_test_runtime::SystemConfig {
			code: avn_test_runtime::WASM_BINARY
				.expect("WASM binary was not built, please build it!")
				.to_vec(),
		},
		balances: avn_test_runtime::BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|(k, a)| (k, a)).collect(),
		},
		parachain_info: avn_test_runtime::ParachainInfoConfig { parachain_id: id },
		session: avn_test_runtime::SessionConfig {
			keys: candidates
				.iter()
				.cloned()
				.map(|(acc, aura, audi, imon, avn)| {
					(
						acc.clone(),                             // account id
						acc,                                     // validator id
						avn_session_keys(aura, audi, imon, avn), // session keys
					)
				})
				.collect(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		assets: Default::default(),
		ethereum_events: EthereumEventsConfig {
			validator_manager_contract_address: avn_eth_contract.clone(),
			lifting_contract_address: avn_eth_contract.clone(),
			nft_t1_contracts: nft_eth_contracts,
			processed_events: vec![],
			lift_tx_hashes,
			quorum_factor: QUORUM_FACTOR,
			event_challenge_period,
		},
		ethereum_transactions: EthereumTransactionsConfig {
			_phantom: Default::default(),
			get_publish_root_contract: avn_eth_contract.clone(),
		},
		validators_manager: ValidatorsManagerConfig {
			validators: candidates
				.iter()
				.map(|x| x.0.clone())
				.zip(eth_public_keys.iter().map(|pk| pk.clone()))
				.collect::<Vec<_>>(),
		},
		authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
		aura: Default::default(),
		aura_ext: Default::default(),
		im_online: ImOnlineConfig { keys: vec![] },
		parachain_system: Default::default(),
		parachain_staking: ParachainStakingConfig {
			candidates: candidates
				.iter()
				.cloned()
				.map(|(acc, _, _, _, _)| (acc, COLLATOR_DEPOSIT))
				.collect(),
			nominations: vec![],
			min_collator_stake: COLLATOR_DEPOSIT,
			min_total_nominator_stake: 10 * AVT,
			delay: 2,
		},
		polkadot_xcm: avn_test_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
		sudo: SudoConfig { key: Some(sudo_account) },
		summary: SummaryConfig { schedule_period, voting_period },
		token_manager: TokenManagerConfig {
			_phantom: Default::default(),
			lower_account_id: H256(hex!(
				"000000000000000000000000000000000000000000000000000000000000dead"
			)),
			// Address of AVT contract
			avt_token_contract,
		},
	}
}
