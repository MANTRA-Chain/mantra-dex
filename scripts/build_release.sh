#!/usr/bin/env bash
set -e

projectRootPath=$(realpath "$0" | sed 's|\(.*\)/.*|\1|' | cd ../ | pwd)

# Displays tool usage
function display_usage() {
	echo "Release builder"
	echo -e "\nUsage:./build_release.sh\n"
}

docker_options=(
	--rm
	-v "$projectRootPath":/code
	--mount type=volume,source="$(basename "$projectRootPath")_cache",target=/target
	--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry
)

# if the operative system is running arm64, append -arm64 to the optimizer. Otherwise not
arch=$(uname -m)

if [[ "$arch" == "aarch64" || "$arch" == "arm64" ]]; then
	docker_command=("docker" "run" "${docker_options[@]}" "cosmwasm/optimizer-arm64:0.16.0")
else
	docker_command=("docker" "run" "${docker_options[@]}" "cosmwasm/optimizer:0.16.0")
fi

echo "${docker_command[@]}"

# Execute the Docker command
"${docker_command[@]}"

# Check generated wasm file sizes
$projectRootPath/scripts/check_artifacts_size.sh

# Get artifacts versions
$projectRootPath/scripts/get_artifacts_versions.sh
