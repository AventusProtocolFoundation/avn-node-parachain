use crate::chain_spec::stable::ChainSpec;

/// Configuration of AvN parachain on Polkadot network (Mainnet).
pub(crate) fn avn_polkadot_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/avn_polkadot_raw.json")[..])
}

/// Configuration of AvN parachain on Rococo network (Public Testnet).
#[cfg(not(feature = "rococo-spec-build"))]
pub(crate) fn avn_rococo_config_v5() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/avn_rococo_v5_raw.json")[..])
}

#[cfg(feature = "rococo-spec-build")]
pub(crate) fn avn_rococo_config_v5() -> Result<ChainSpec, String> {
	build_rococo_spec::avn_rococo_config_v5()
}

#[cfg(feature = "rococo-spec-build")]
mod build_rococo_spec {
	use crate::chain_spec::{
		avn_chain_properties, constants::*, AuraId, AuthorityDiscoveryId, AvnId, ChainSpec,
		ChainType, EthPublicKey, Extensions, ImOnlineId,
	};

	use crate::chain_spec::avn_runtime::testnet_genesis;
	use hex_literal::hex;
	use node_primitives::AccountId;
	use sp_core::{crypto::UncheckedInto, ecdsa, ByteArray, H160};

	pub(crate) fn avn_rococo_config_v5() -> Result<ChainSpec, String> {
		let rococo_parachain_id: u32 = 2056;
		Ok(ChainSpec::from_genesis(
			// Name
			"AvN Rococo",
			// ID
			"avn_rococo_v5",
			ChainType::Live,
			move || {
				testnet_genesis(
					// initial collators.
					avn_rococo_authorities_keys(),
					// endowed accounts
					vec![
						// SUDO account
						(
							hex![
								"e05fc2d5ee74aa771678cab3bbb9ba5a11815f85e349566bca12c028132d3f00"
							]
							.into(),
							AVT_ENDOWMENT,
						),
						// Validator accounts
						(
							hex![
								"4a9a2c1b8aa9d2a0cc948ae1c911e0640642f02dd638d32aa0d359899d69f63c"
							]
							.into(),
							AVT_ENDOWMENT,
						),
						(
							hex![
								"3e3640703ce7db67f02bbbc717c4c46a067bc03db14982efff562d7fdee0977e"
							]
							.into(),
							AVT_ENDOWMENT,
						),
						(
							hex![
								"b2769f978892850e76283e784c9d3896720da08e38ddb75b813c9316ce516c10"
							]
							.into(),
							AVT_ENDOWMENT,
						),
						(
							hex![
								"b8b982a45579f8d5af80fc94fe5bea799e094418c3acb87afc550d090cd47a42"
							]
							.into(),
							AVT_ENDOWMENT,
						),
						(
							hex![
								"708d4d3e9252c88ce8a5b63c85b2d3cb17b8ff77b18b95e855744f8f2393bf4b"
							]
							.into(),
							AVT_ENDOWMENT,
						),
					],
					rococo_parachain_id.into(),
					// SUDO account
					hex!["e05fc2d5ee74aa771678cab3bbb9ba5a11815f85e349566bca12c028132d3f00"].into(),
					// AVT contract
					H160(hex!("97d9b397189e8b771FfAc3Cb04cf26C780a93431")),
					// AVN contract
					H160(hex!("b8877fB8C687561F7B01872C0D9d373FFfF0Fdb7")),
					// nft contracts
					vec![],
					avn_rococo_testnet_ethereum_public_keys(),
					vec![
						hex!("c0dba9ff6de13d485c0fbf83f3640d28a22e9aadd87993ae93a7804d8d367681")
							.into(),
						hex!("3666357d7a7dc9a55fe448f3d8d982a589a4d6db6d2b7622cdb47c12e72c4626")
							.into(),
					],
					NORMAL_EVENT_CHALLENGE_PERIOD,
					EIGHT_HOURS_SCHEDULE_PERIOD,
					NORMAL_VOTING_PERIOD,
				)
			},
			// Bootnodes
			Vec::new(),
			// Telemetry
			None,
			// Protocol ID
			Some("avn_rococo_v4"),
			// Fork ID
			None,
			// Properties
			avn_chain_properties(),
			// Extensions
			Extensions {
				relay_chain: "rococo".into(), // You MUST set this to the correct network!
				para_id: rococo_parachain_id,
			},
		))
	}

	#[rustfmt::skip]
    fn avn_rococo_authorities_keys(
    ) -> Vec<(AccountId, AuraId, AuthorityDiscoveryId, ImOnlineId, AvnId)> {
        let initial_authorities: Vec<(AccountId, AuraId, AuthorityDiscoveryId, ImOnlineId, AvnId)> = vec![
            (
                // "5DkXCWvTpBuFgQgc6uB78Wqy6EkSDTrcYEcfQY5JoyPVTdUk"
                hex!["4a9a2c1b8aa9d2a0cc948ae1c911e0640642f02dd638d32aa0d359899d69f63c"].into(),
                // "5Esuos3iXWfsSDgnYpavX2VKF95oujwVgWRg33r7qmrWqZFR"
                hex!["7c793bd43c7a08fda5eb6301b67cecff639eb6a23b7d39aede9abdc1b666c465"].unchecked_into(),
                // "5Eee1FQfhMAzX899ihY1DpzR5X3VvgukwJdjEiEuVc9nmPr3"
                hex!["7259d7ed21015722686063da4a4abc3299a9b7aa70389afa870658d9cf29c77e"].unchecked_into(),
                // "5F4TWhsiFMtpbgbDZWgBxZmVqgzDSw2ffm1c6oPYpqqdqhrG"
                hex!["848469db28136f96fa85a963c6e3f3ae878b2ddfebd36aa5e2c01d105295754e"].unchecked_into(),
                // "5EtWHJtM8Z6DVAAwwpGd6a31wHgvg2scF6C26X381uMVtBk5"
                hex!["7ced47e4e8dc2d1bc4d77d01e1e7778fccf50cdff177d06cb3de048bc952b80d"].unchecked_into(),
            ),
            (
                // "5DUGwbbtq4a5ZktGXVwgrvovWLfvVHJRPextENWu7MNs3ysY"
                hex!["3e3640703ce7db67f02bbbc717c4c46a067bc03db14982efff562d7fdee0977e"].into(),
                // "5GgRf4oAdEocs2Tz5C4ae8UBQue8RhbKymSY7pdRnzsJtXKL"
                hex!["cc2f3bf2212fddcd65f535f67776e20964eddbfd93d483d6716757ff3e82b734"].unchecked_into(),
                // "5HZ6TdciCdEFSyCHNNi5nos2h5xCqg6BoJY1oRk3f14VRybn"
                hex!["f2d4205f1227f84b6f96d6833482954c619a454d14721194e1d424de1ca2927e"].unchecked_into(),
                // "5GGbkXHnoxJSbPDqq467BayvPhfNEsptA2ftpGHLgrGyhSDc"
                hex!["ba0352fcc3003567d2e6c299f84a02e33b7144b6dbbddb7ba5b0d6a9fae48505"].unchecked_into(),
                // "5GgauAGuaFYZG91mmPPJ2W8ADnqgnwQ8Mg7tXZ5E1QHAFGCH"
                hex!["cc4e595197accbb18e307c73122bdaeb4840e5055c9c25e465dc96fef163a031"].unchecked_into(),
            ),
            (
                // "5G6hcs5EaFsCH48Es7xUUQeaC9LJ61z8byX5CnvTqcmCNFXH"
                hex!["b2769f978892850e76283e784c9d3896720da08e38ddb75b813c9316ce516c10"].into(),
                // "5EZCAbnRwmZ9a7oPZ6U2Yy4cbmXSTQ7sKWUPiAPzg9LyPf6g"
                hex!["6e32a2a88c74c64326ca92182f0a7d8d29d2808f16e2e86729d6068e9bcdf673"].unchecked_into(),
                // "5CV4ah2Wp6pkZ2AvivMGVU6k6DXuALyTZRxYQnkSHFWXFxKm"
                hex!["1293a797b32e75e0b6272e44f5d83d4d15fa0b4d300a730ae49a54f2a78d9211"].unchecked_into(),
                // "5FxNJRyaSWrm97uMbaaMTBJRgGWBUdyABAcTHiGj87RJSCeb"
                hex!["ac1ba171b2630042c67e4769848b5ba45d2915f47f0e402a29c6788ffde72945"].unchecked_into(),
                // "5F4icb3MT65Y11SfqNo3Wp3GMSidSVLPQHek8gThYRDs5xLY"
                hex!["84b73fd8bd6d8cf990ef1abbacf0ba363aaa650f908467f46eb2c327c4437362"].unchecked_into(),
            ),
            (
                // "5GEumxnr6Mwu66Zt1ed6zi1NcD6SWrpYDHTWpShMnV5XoHja"
                hex!["b8b982a45579f8d5af80fc94fe5bea799e094418c3acb87afc550d090cd47a42"].into(),
                // "5Ehu5jVWGRjoGkAL3bwjFgiDxiq4CV7BN5UudneHnX2XJvPx"
                hex!["74d6559db2160485aef028ac2d3af4c29a86e0296ce405a2384503e94a92495b"].unchecked_into(),
                // "5GW6sMvbi6xU7iupS3p2prn2NEWsfaoMXLv5hq8VPx33JKG7"
                hex!["c44f88b67481cb0cb6c23e230f65c013f298008154a0d67498b4eb033b587d0a"].unchecked_into(),
                // "5FR66hnzQdSD73pavCCMvrTnh9o2wwoyp1V1wVhLwJMxqPTt"
                hex!["94407b4de17a1c468dd8abd58b2cf0ffff07f2c0784fd331274a2626ea66460a"].unchecked_into(),
                // "5DUQQmDjMrFCHXkbyAFxxefsWCoXSGeRw11sezEH4CVHciPW"
                hex!["3e4f647b6ccf0b3cf31fbb729c9e3bef5b4398bced0bc0f08e8ca76989290b0f"].unchecked_into(),
            ),
            (
                // "5EcHCJMY9pZFokHDLRr1xHAQrUJBiuyyP72LfPu4ZauyRqhq"
                hex!["708d4d3e9252c88ce8a5b63c85b2d3cb17b8ff77b18b95e855744f8f2393bf4b"].into(),
                // "5GqJtGxordGUkL64dWC7dytMmWA83hf8iUHz1ezfu7Tqgv8t"
                hex!["d2f5a5386bf3008c5eb208901b445de33c6389364074f9b08233340922c3ed16"].unchecked_into(),
                // "5HKXdy53cg2xe5kT4b9C2JCpd1Mr6bCff8jTnYvHvnmXLap6"
                hex!["e87b72ba4fe30086b15948b9f969cf57778bde75ab3a77411769db54b1a61a50"].unchecked_into(),
                // "5DCJrKa5fXSuvgWRJVnST9chgrxwevNAsVjm75JX9SPkoJDi"
                hex!["3208c0690506b5aca8a8b956e11958ad91588282b0dc4587580d535e33063123"].unchecked_into(),
                // "5En3hWNTTDSkLC4JX44uGNrHHMiRnuPbzn2K45Lmj2k5WGgn"
                hex!["7800529feb65b072616b325991140874074dd58439bdd45099b8d8457b7f3a59"].unchecked_into(),
            ),
        ];
        return initial_authorities;
    }

	fn avn_rococo_testnet_ethereum_public_keys() -> Vec<EthPublicKey> {
		return vec![
			// 0x29E63a286F37559f9BD6e7aC5fC47FA5E9fA54F1
			ecdsa::Public::from_slice(&hex![
				"021f21d300f707014f718f41c969c054936b7a105a478da74d37ec75fa0f831f87"
			])
			.unwrap(),
			// 0x6C233739E36Ec19069Bceab5eeeBD8ecE8bc8411
			ecdsa::Public::from_slice(&hex![
				"024b3e07593467cf4ac01f9c48a0a1e8f4d7056d79a322c8cfa13226c3333f5acd"
			])
			.unwrap(),
			// 0x3153795e441493B0E2dF285d52B945F2fdD93A7b
			ecdsa::Public::from_slice(&hex![
				"0317c85eb98798941224de6effaf37871e66ea255976df07ae2c4bf287cb155802"
			])
			.unwrap(),
			// 0xC53f162E5e7Aea001fE7c1cBcd4A793f6b8560C9
			ecdsa::Public::from_slice(&hex![
				"02727125c6793951ec4733a3f09ffcd011b721af4696e7ed290ee3428985ccbf22"
			])
			.unwrap(),
			// 0x835d71301Cf490401EA0c82eca8532ad0b08630a
			ecdsa::Public::from_slice(&hex![
				"028abc959e22fab9d554548b685cb03da0b08479d611c96c57c8cd2445ad6d12b2"
			])
			.unwrap(),
		]
	}
}
