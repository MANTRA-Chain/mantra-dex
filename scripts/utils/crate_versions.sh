#!/usr/bin/env bash

project_root_path=$(realpath "$0" | sed 's|\(.*\)/.*|\1|' | cd ../ | pwd)
cargo_lock_file="$project_root_path/Cargo.lock"

# Extracts crate names and versions
extract_crates() {
	cargo_lock_file="$1"
	filter_mode="$2"

	# Extract crate names and versions using awk
	awk -F '"' '/^name = / { name=$2 } /^version = / { version=$2 } name && version { printf "%s (%s)\n", name, version; name=""; version="" }' "$cargo_lock_file" |
		filter_crates "$filter_mode"
}

# Filters crate names based on a preset array
filter_crates() {
	filter_mode="$1"
	# No filtering, pass through all crate names
	cat -
}

if [ "$1" = "-h" ]; then
	echo -e "\nUsage: crate_versions.sh\n"
	echo "List names and versions of cargo crates in the project."
	echo ""
	echo "Options:"
	echo -e "-h        Show this help message\n"
	exit 0
fi

cargo build

filter_mode="all"

# Call the extract_crates function with the filter mode
crates=$(extract_crates "$cargo_lock_file" "$filter_mode")

echo -e "\nCrate names and versions:\n"

echo -e "$crates\n"
