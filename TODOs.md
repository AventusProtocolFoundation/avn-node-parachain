# Avn-Node pallet and code migration TODOs

- Review adding `#[pallet::without_storage_info]` in `#[pallet::pallet]`
    - avn-pallet
- Local storage locks
    - primitives avn_common
        - Setting expiring locks
        - Unit tests pass
    - pallet avn
        - avn-port is read when set as intended
    - pallet avn-finality-tracker
        - Ensure that the last finalised block submission is stored/retrieved as expected
    - pallet-ethereum-events
        - local storage, validated events
        - genesis config and on_genesis function
- Review any "TODO: [STATE MIGRATION]" comments and address them as part of the storage migration work

- Runtime
    - Review the offenders identification implementation
    - Use validators manager as the offence enforcer