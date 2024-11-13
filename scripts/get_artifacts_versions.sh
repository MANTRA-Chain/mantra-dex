#!/usr/bin/env bash
set -e

project_root_path=$(realpath "$0" | sed 's|\(.*\)/.*|\1|' | cd ../ | pwd)

if [ "$1" != "--skip-verbose" ]; then
	echo -e "\nGetting artifacts versions...\n"
fi

echo -e "\033[1mContracts:\033[0m"
for artifact in artifacts/*.wasm; do
	artifact="${artifact%-*}"
	artifact=$(echo $artifact | sed 's/_/-/g')
	contract_path=$(find "$project_root_path" -iname $(cut -d . -f 1 <<<$(basename $artifact)) -type d)
	version=$(cat ''"$contract_path"'/Cargo.toml' | awk -F= '/^version/ { print $2 }')
	version="${version//\"/}"

	printf "%-20s %s\n" "$(basename $artifact)" ": $version"
done
echo -e "\n\033[1mPackages:\033[0m"

for package in packages/*; do
	version=$(cat ''"$package"'/Cargo.toml' | awk -F= '/^version/ { print $2 }')
	version="${version//\"/}"

	printf "%-20s %s\n" "$(basename $package)" ": $version"
done
