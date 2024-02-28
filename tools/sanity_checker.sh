#!/bin/bash
set -e

usage() {
  echo -e "\n\
    Script usage:\n\
      Test our code - not all of Substrate - optionally with coverage \n\
    Run as \n\
      ./test-our-code.sh [commands] \n\
    Commands: \n\
      test    (default) Runs cargo test on the targets. Default action if no command is set.\n\
      grcov   Run tests with grcov coverage\n\
      tarp    Run tests with tarpaulin coverage\n\
      bench   Runs tests and benchmarks\n\
      fmt     Runs cargo fmt on the targets\n\
      todo    Print all the TODOs in these directories\n\
  \n"
  exit
}

runCommand() {
  echo
  echo "=== $1 ==="
  pushd $1
  $CMD
  popd
}

run() {
  # TODO restore these when integrated again.
  # runCommand bin/node/cli/avn-service
  # runCommand bin/node/rpc
  run_pallets_that_allow_benchmarks
}

run_pallets_that_allow_benchmarks() {
  runCommand pallets/avn-finality-tracker
  runCommand pallets/avn-proxy
  runCommand pallets/avn-offence-handler
  runCommand pallets/ethereum-events
  runCommand pallets/ethereum-transactions
  runCommand pallets/nft-manager
  runCommand pallets/summary
  runCommand pallets/token-manager
  runCommand pallets/validators-manager
}

REPOSITORY_ROOT=$(dirname $(dirname $(readlink -f $0 || realpath $0)))
pushd $REPOSITORY_ROOT

if [ "$1" == "tarp" ]; then
  # See https://crates.io/crates/cargo-tarpaulin
  echo "*** WITH TARPAULIN"
  export CMD='cargo tarpaulin -b -o Html'
  run
  echo "*** See, eg pallets/ethereum-events/tarpaulin-report.html\#pallets/ethereum-events/src for results"
elif [ "$1" == "grcov" ]; then
    # See https://lib.rs/crates/grcov
    echo "*** WITH GRCOV"
    export CARGO_INCREMENTAL=0
    export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
    export RUSTDOCFLAGS="-Cpanic=abort"
    export CMD='cargo +nightly test'
    run
    grcov ./target/debug/ -s . -t html --llvm --branch --ignore-not-existing -o ./target/debug/coverage/
    echo "*** See target/debug/coverage/ for results"
elif [ "$1" == "bench" ]; then
  export CMD='cargo test --features runtime-benchmarks -- benchmarking'
  run_pallets_that_allow_benchmarks
elif [ "$1" == "todo" ]; then
  export CMD='git grep TODO'
  run
elif [ "$1" == "fmt" ]; then
  export CMD='cargo fmt --check'
  run
elif [ "$1" == "" ] || [ "$1" == "test" ]; then
  export CMD='cargo test'
  run
else
  usage
fi

popd
