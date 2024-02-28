# AvN Node Parachain
[![avn-node-parachain test & build](https://github.com/Aventus-Network-Services/avn-node-parachain/actions/workflows/ci-cron.yml/badge.svg?event=schedule)](https://github.com/Aventus-Network-Services/avn-node-parachain/actions/workflows/ci-cron.yml)

A [Cumulus](https://github.com/paritytech/cumulus/) FRAME-based Substrate Node compatible with the AvN parachain.

This project is originally a fork of the
[Substrate Node Template](https://github.com/substrate-developer-hub/substrate-node-template)
modified to include dependencies required for registering this node as a **parathread** or
**parachain** to a **relay chain**.

The stand-alone version of this template is hosted on the
[Substrate Devhub Parachain Template](https://github.com/substrate-developer-hub/substrate-parachain-template/)
for each release of Polkadot. It is generated directly to the upstream
[Parachain Template in Cumulus](https://github.com/paritytech/cumulus/tree/master/parachain-template)
at each release branch using the
[Substrate Template Generator](https://github.com/paritytech/substrate-template-generator/).

👉 Learn more about parachains [here](https://wiki.polkadot.network/docs/learn-parachains), and
parathreads [here](https://wiki.polkadot.network/docs/learn-parathreads).


🧙 Learn about how to use this template and run your own parachain testnet for it in the
[Devhub Cumulus Tutorial](https://docs.substrate.io/tutorials/v3/cumulus/start-relay/).

## Building the project
*Based on [Polkadot Build Guide](https://github.com/paritytech/polkadot#building)*

First [Install Rust](https://www.rust-lang.org/tools/install):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

If you already have Rust installed, make sure you're using the latest version by running:

```bash
rustup update
```

Once done, finish installing the support software:

```bash
# Additional rust targets
rustup target add wasm32-unknown-unknown --toolchain nightly-2020-10-18
# Additional OS dependencies
sudo apt install build-essential git clang libclang-dev pkg-config libssl-dev
```

Build the client by cloning this repository and running the following commands from the root
directory of the repo:

```bash
git checkout <latest tagged release>
cargo build --release
```

### Enabling avn optional features

[The Cargo Book - Features](https://doc.rust-lang.org/cargo/reference/features.html) section explains how features and feature flags can be used in cargo to enable additional features when building a project.

```bash
# single feature
cargo build --features feature_name
# multiple features
cargo build --features feature_name_1,feature_name_2
```

#### Activating the Test Runtime

The avn test runtime is an independent runtime that integrates the newest features of AvN, currently undergoing development and testing. By default, the test runtime is not built, but you can enable it using one of the following feature flags:

- `avn-test-runtime`: Enables the compilation and chainspecs that utilize the test runtime
- `test-native-runtime`: Switches the native runtime of the node to use the test runtime. This feature implies the `avn-test-runtime` feature.

#### Building the Rococo Specification

By default, the rococo chain specification utilizes a prebuilt [chainspec raw](./node/src/chain_spec/res/avn_rococo_v5_raw.json) file, ensuring consistent chain configuration output. However, you can regenerate the rococo spec by using the `rococo-spec-build` flag, which recreates the spec from the chain_spec module using the native avn runtime. This flag is particularly useful when you need to update the rococo parachain to reset the network.

### Using devcontainers & vscode extensions

#### Prerequisites
- Docker 20.10.17 or newer
- vscode with the following plugin
    - [Remote Development](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.vscode-remote-extensionpack)
    - [Remote - Containers](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)

Once prerequisites are installed, open the project in vscode and press F1 to bring up the Command Palette and type in Remote-Containers for a full list of commands. Select then `Reopen in container` or `Rebuild and reopen in container`. This will reopen the project and build the container that will be used to run commands and build the code, which can cause a small delay the first time you launch it (or every time you rebuild the devcontainer). You can use then the vscode console to build the code.

### Building the docker image
```sh
# Builds the docker image with the build artefacts under target/release
docker build . --tag avn-node-parachain:latest
```

## Logs

When running a node, various log messages are displayed in the output. Each log has a level sensitivity, such as `error`, `warning`, `info`, `debug`, or `trace` and is associated with a specific target.

The following CLI options can be used to configure logging:
```
  -l, --log <LOG_PATTERN>...
          Sets a custom logging filter. Syntax is `<target>=<level>`, e.g. -lsync=debug
  --detailed-log-output
      Enable detailed log output
```

Enabling the `detailed-log-output` flag provides more comprehensive log information, including the log target, log level, and the name of the emitting thread. If no target is specified, the name of the module will be used.

Here are some examples of log statements and their corresponding outputs:

```Rust
log::info!(target: "aventus", "💾 Sample log");
```
Output:

```console
2023-05-15 08:00:00 💾 Sample log
```
Output with detailed log output enabled:
```console
2023-05-15 08:00:00  INFO main aventus: [Parachain] 💾 Sample log
```
During node execution, you have the flexibility to modify the log level for all or specific targets using the following command line parameters:
```
# Setting the log level to all targets
-ldebug
--log debug

# Setting the log level to specific targets.
-ltxpool=debug -lsub-libp2p=debug
```
There are numerous log targets available, and you can discover more by utilizing the detailed-log-output parameter or by referring to the code. However, for convenience, here are some commonly used log targets:

- txpool
- avn-service
- sub-libp2p
- sub-authority-discovery
- parachain::collator-protocol
- parachain::validator-discovery
- gossip
- peerset
- cumulus-collator
- db
- executor
- wasm-runtime
- sync
- offchain-worker::http
- state-db
- state

*Please note that this list is not exhaustive*.

### Log output manipulation using environment variables

Alternatively, you can use the `RUST_LOG` environment variable to specify the desired log level per module. Substrate utilizes the [log crate](https://github.com/rust-lang/log) and [env_logger crate](https://docs.rs/env_logger/latest/env_logger/) for its internal logging implementation. These crates provide a flexible and configurable way to manage log output through the use of the `RUST_LOG` environment variable.

- [Substrate - Debug](https://docs.substrate.io/test/debug/)
- [Rust - Enabling logs per module](https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/config_log.html)


## CI
The project will produce build artefacts when:
- commits are pushed to master
- a release is created

At the moment only docker images are build and they are pushed in the `189013141504.dkr.ecr.eu-west-2.amazonaws.com/avn/avn-node-parachain` ECR repository.

To pull the image you will need to authenticate with the docker registry. Provided you have the appropriate aws credentials configured, run:
```sh
# authentication with the registry
aws ecr get-login-password --region eu-west-2 | docker login --username AWS --password-stdin 189013141504.dkr.ecr.eu-west-2.amazonaws.com
```
Most tools, like `docker compose` are now able to pull the images they need. Should you want to manually pull the image, run:
```sh
# authentication with the registry
aws ecr get-login-password --region eu-west-2 | docker login --username AWS --password-stdin 189013141504.dkr.ecr.eu-west-2.amazonaws.com
# retrieve the image
docker pull 189013141504.dkr.ecr.eu-west-2.amazonaws.com/avn/avn-node-parachain:latest
```

## Original Rococo spec generation
This section is for documentation purposes and to capture the way the rococo chainspec was created.
Do a release build with `rococo-spec-build` feature enabled.
```sh
# build command
cargo build --release --features rococo-spec-build
# avn parachain rococo chainspec creation
target/release/avn-parachain-collator build-spec --chain rococo --raw --disable-default-bootnode > node/src/chain_spec/res/rococo-avn.json
```
Edit manually the file to add the bootnodes:
```diff
--- a/node/src/chain_spec/res/rococo-avn.json
+++ b/node/src/chain_spec/res/rococo-avn.json
@@ -2,7 +2,16 @@
   "name": "AvN Rococo",
   "id": "avn_rococo",
   "chainType": "Live",
-  "bootNodes": [],
+  "bootNodes": [
+    "/ip4/<ip>/tcp/40333/p2p/<node-id>"
+  ],
```
## Rococo spec v3 generation
v0.9.1 binary was used so the chain uses exactly the same runtime as the one submitted on polkadot auction.
```sh
# Create a human readable version of the chainspec
./avn-parachain-collator build-spec --chain rococo --disable-default-bootnode > node/src/chain_spec/res/avn_rococo_v3_human.json
```
Manual edit of the file, updating the parachain id to 2056 and setting 10 AVT to `sudo` and `collator` accounts
```diff
@@ -11,7 +11,7 @@
     "tokenSymbol": "AVT"
   },
   "relay_chain": "rococo",
-  "para_id": 4095,
+  "para_id": 2056,
   "codeSubstitutes": {},
   "genesis": {
     "runtime": {
@@ -20,10 +20,35 @@
       },
       "parachainSystem": null,
       "parachainInfo": {
-        "parachainId": 4095
+        "parachainId": 2056
       },
       "balances": {
-        "balances": []
+        "balances": [
+          [
+            "5H8u2iEfeGqj3R6dSdL1hcfWQ4GWtco1HoFiZJDvrCdapvR7",
+            1000000000000000000
+          ],
+          [
+            "5DkXCWvTpBuFgQgc6uB78Wqy6EkSDTrcYEcfQY5JoyPVTdUk",
+            1000000000000000000
+          ],
+          [
+            "5DUGwbbtq4a5ZktGXVwgrvovWLfvVHJRPextENWu7MNs3ysY",
+            1000000000000000000
+          ],
+          [
+            "5G6hcs5EaFsCH48Es7xUUQeaC9LJ61z8byX5CnvTqcmCNFXH",
+            1000000000000000000
+          ],
+          [
+            "5GEumxnr6Mwu66Zt1ed6zi1NcD6SWrpYDHTWpShMnV5XoHja",
+            1000000000000000000
+          ],
+          [
+            "5EcHCJMY9pZFokHDLRr1xHAQrUJBiuyyP72LfPu4ZauyRqhq",
+            1000000000000000000
+          ]
+        ]
       },
       "validatorsManager": {
         "validators": [
```
Then create the raw file:
```sh
./avn-parachain-collator build-spec --chain node/src/chain_spec/res/avn_rococo_v3_human.json --disable-default-bootnode > node/src/chain_spec/res/avn_rococo_v3_raw.json
```
## Performing an upgrade to an avn parachain Network
For now upgrades are only supported via the sudo pallet in the parachain runtime.

[Official guide for upgrades.](https://docs.substrate.io/tutorials/get-started/forkless-upgrade/#authorize-an-upgrade-using-the-sudo-pallet)

To perform the upgrade you will need access to the sudo account and the runtime you wish the chain to upgrade to.

The wasm runtime is generated during the build process under the target/release/wbuild/avn-parachain-runtime/ directory. There is also a compact and compressed version generated:
- avn_parachain_runtime.wasm
- avn_parachain_runtime.compact.wasm
- avn_parachain_runtime.compact.compressed.wasm

You can also use the `avn-parachain-collator` binary from the build you want to upgrade to, and extract the wasm from it:
```sh
# Generating a raw chainspec
avn-parachain-collator build-spec —chain dev > dev-chain.json
avn-parachain-collator build-spec --chain dev-chain.json --raw --disable-default-bootnode > dev-chain-raw.json
# extract the wasm from the genesis confifg
avn-parachain-collator export-genesis-wasm --chain dev-chain-raw.json > upgrade.wasm
# Alternatively for configurations supported by the CLI, you can use this one-liner
avn-parachain-collator export-genesis-wasm --chain dev > upgrade.wasm
```

To perform the upgrade, connect via the polkadot ui to the parachain and navigate to
`sudo > parachainSystem > AuthorizeUpgrade`. Slide the `hash a file` button, and browse to the wasm file you wish to upgrade on your local disc. Submit the sudo action and await. Once accepted you should see in the explorer tab the following event:
```
  parachainSystem.UpgradeAuthorized
    An upgrade has been authorized.
```
Now you can enact the upgrade. Navigate to `sudo > parachainSystem > enactAuthorizedUpgrade`. Slide the `file upload` button, and browse to the wasm file you wish to upgrade on your local disc. Submit the sudo action. Once accepted you should be able to see it in the explorer.

```
parachainSystem.ValidationFunctionApplied
  The validation function was applied as of the contained relay chain block number.
sudo.Sudid
  A sudo just took place. [result]
    sudoResult: Result<Null, SpRuntimeDispatchError>
    Ok
```
## Extracting the relay chain spec

To run a parachain node, you need to provide the relay chain spec that is designated for. To retrieve that, you can use the following commands.

For permanent networks such as rococo, different builds do not change the output.
```sh
# This will spin of a docker instance and a terminal to use the official builds to extract the spec
docker run --rm -it --mount type=bind,source="$(pwd)"/,target=/export-relayer-spec --entrypoint /bin/bash parity/polkadot:v0.9.27
# This example uses rococo, but you can use any of the options polkadot provides, for example versi-local or dev.
polkadot build-spec --chain rococo --raw > /export-relayer-spec/rococo-raw.json
exit
```
