#!/bin/bash
# A script that downloads and extracts avn-node and wasm file from a docker image.

# Prerequistives: Configured credentials with AWS ECR in public testnet account

function cleanup {
    rm -rf ./docker_cid
    echo "Stopping container"
    docker stop ${container_id}
}

trap cleanup EXIT

usage() {
  echo -e "\n\
    Script usage: \n\
      This script extracts avn binaries from a docker image. \n\
    Run as \n\
      ./extract_avn_binaries.sh [options] \n\
    Options: \n\
      -h, --help        Displays usage information. \n\
      -t, --tag         the docker tag to use. \n\
      -p, --profile     the aws profile to use. \n\
  \n"
  exit
}

while [[ "$#" -gt 0 ]]; do case $1 in
  -t|--tag) DOCKER_TAG="$2"; shift;;
  -p|--profile) AWS_PROFILE="--profile $2"; shift;;
  *) echo "Unknown parameter passed: $1"; usage; exit 1;;
esac; shift; done

if [[ -z "${DOCKER_TAG+x}" ]]; then usage; fi

DOCKER_REPO="189013141504.dkr.ecr.eu-west-2.amazonaws.com/avn/avn-node-parachain"

# Loging to ECR and refresh login credentials
aws ecr get-login-password --region eu-west-2 ${AWS_PROFILE} | docker login --username AWS --password-stdin ${DOCKER_REPO}

docker pull ${DOCKER_REPO}:${DOCKER_TAG}

docker run --rm --cidfile ./docker_cid --name avn-tier2-binary-extract -d ${DOCKER_REPO}:${DOCKER_TAG} --chain=local  
container_id=$(cat ./docker_cid)
echo "Started container ${container_id}"

# COPY --from=builder /avn-node-parachain/target/release/avn-parachain-collator /usr/local/bin
# COPY --from=builder /avn-node-parachain/target/release/wbuild/parachain-template-runtime/parachain_template_runtime.wasm /avn/wbuild/

docker cp ${container_id}:/usr/local/bin/avn-parachain-collator ./
docker cp ${container_id}:/avn/wbuild/avn_parachain_runtime.compact.compressed.wasm ./
docker cp ${container_id}:/avn/wbuild/avn_parachain_test_runtime.compact.compressed.wasm ./

echo "-------------------------------"
echo "Files copied. sha256 checksums:"
sha256sum avn-parachain-collator avn_parachain_runtime.compact.compressed.wasm avn_parachain_test_runtime.compact.compressed.wasm
echo "-------------------------------"
