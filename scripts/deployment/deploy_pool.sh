#!/usr/bin/env bash
set -e

deployment_script_dir=$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
project_root_path=$(realpath "$0" | sed 's|\(.*\)/.*|\1|' | cd ../ | pwd)

# Displays tool usage
function display_usage() {
	echo "MANTRA Dex Pool Deployer"
	echo -e "\nUsage:./deploy_pool.sh [flags]. Does not support stable pools just yet.\n"
	echo -e "Available flags:\n"
	echo -e "  -h \thelp"
	echo -e "  -c \tThe chain where you want to deploy (mantra|mantra-testnet)"
	echo -e "  -p \tPool configuration file to get deployment info from."
	echo -e "  -a \tThe amount to provide liquidity with, if any, comma separated, for denom 0 and denom 1. e.g. 1000,2000 would mean 1000denom0,2000denom1"
}

# Reads a pool config file, like the follow:
#
#{
#  "protocol_fee": "0.001",
#  "swap_fee": "0.002",
#  "burn_fee": "0",
#  "pool_type": "constant_product",
#  "pool_identifier": "pool_identifier",
#  "assets": [
#    {
#      "denom": "uom",
#      "decimals": 6
#    },
#    {
#      "denom": "uusdc",
##     "decimals": 6
#    }
#  ]
#}
function read_pool_config() {
	if [ $# -eq 1 ]; then
		local pool=$1
	else
		echo "read_pool_config requires a pool config file"
		exit 1
	fi

	mapfile -t assets < <(jq -c '.assets[]' <$pool)
	protocol_fee=$(jq -r '.protocol_fee' $pool)
	swap_fee=$(jq -r '.swap_fee' $pool)
	burn_fee=$(jq -r '.burn_fee' $pool)
	pool_type=$(jq -r '.pool_type' $pool)
	pool_identifier=$(jq -r '.pool_identifier' $pool)
}

function create_pool() {
	mkdir -p $project_root_path/scripts/deployment/output
	output_file=$project_root_path/scripts/deployment/output/"$CHAIN_ID"_pools.json
	deployment_file=$project_root_path/scripts/deployment/output/"$CHAIN_ID"_mantra_dex_contracts.json

	if [[ ! -f "$output_file" ]]; then
		# create file to dump results into
		echo '{"pools": []}' | jq '.' >$output_file
	fi

	pool_manager_addr=$(jq -r '.contracts[] | select (.wasm == "pool_manager.wasm") | .contract_address' $deployment_file)

	label=""

	for asset in "${assets[@]}"; do
		# build the label for the output file
		denom=$(echo $asset | jq -r '.denom')
		if [[ "$denom" == ibc/* ]]; then
			local subdenom=$($BINARY q ibc-transfer denom-trace $denom --node $RPC -o json | jq -r '.denom_trace.base_denom | split("/") | .[-1]')
		elif [[ "$denom" == factory/* ]]; then
			local subdenom=$(basename "$denom")
		else
			local subdenom=$denom
		fi

		label+=$subdenom
		label+="-"

		asset_denoms+=($denom)

		decimals=$(echo $asset | jq -r '.decimals')
		asset_decimals+=($decimals)
	done

	create_pool_msg='{
                     "create_pool": {
                       "asset_denoms":["'${asset_denoms[0]}'","'${asset_denoms[1]}'"],
                       "asset_decimals": [
                         '${asset_decimals[0]}',
                         '${asset_decimals[1]}'
                       ],
                       "pool_fees": {
                         "protocol_fee": {
                           "share": "'$protocol_fee'"
                         },
                         "swap_fee": {
                           "share": "'$swap_fee'"
                         },
                         "burn_fee": {
                           "share": "'$burn_fee'"
                         },
                         "extra_fees": []
                       },
                       "pool_type": "'$pool_type'",
                       "pool_identifier": "'$pool_identifier'"
                     }
                   }'

	echo -e "\e[1;31m⚠️ WARNING ⚠️️\e[0m"

	echo -e "\e[1;32mCreating pool with the following configuration:\e[0m"
	echo -e "\e[1;32mAsset 0: ${asset_denoms[0]} - decimals: ${asset_decimals[0]}\e[0m"
	echo -e "\e[1;32mAsset 1: ${asset_denoms[1]} - decimals: ${asset_decimals[1]}\e[0m"
	echo -e "\e[1;32mPool identifier: $pool_identifier\e[0m"
	echo -e "\e[1;32mProtocol fee: $protocol_fee\e[0m"
	echo -e "\e[1;32mSwap fee: $swap_fee\e[0m"
	echo -e "\e[1;32mBurn fee: $burn_fee\e[0m"

	echo -e "\nDo you want to proceed? (y/n)"
	read proceed

	if [[ "$proceed" != "y" ]]; then
		echo "Pool deployment cancelled..."
		exit 1
	fi

	local config_query='{"config": {}}'
	local pool_creation_fee=$($BINARY q wasm contract-state smart $pool_manager_addr "$config_query" --node $RPC -o json | jq -r '.data.pool_creation_fee')
	local token_factory_fee=$($BINARY q tokenfactory params --node $RPC -o json | jq -r '.params.denom_creation_fee[0]')

	pool_creation_fee_amount=$(echo $pool_creation_fee | jq -r '.amount')
	pool_creation_fee_denom=$(echo $pool_creation_fee | jq -r '.denom')
	token_factory_fee_amount=$(echo $token_factory_fee | jq -r '.amount')
	token_factory_fee_denom=$(echo $token_factory_fee | jq -r '.denom')

	if [ "$pool_creation_fee_denom" = "$token_factory_fee_denom" ]; then
		# Denominations are the same, add amounts together
		total_amount=$(echo $pool_creation_fee_amount + $token_factory_fee_amount | bc)
		amount="$total_amount$pool_creation_fee_denom"
	else
		# Denominations are different, concatenate amounts with denominations
		amount="$pool_creation_fee_amount$pool_creation_fee_denom,$token_factory_fee_amount$token_factory_fee_denom"
	fi

	local res=$($BINARY tx wasm execute $pool_manager_addr "$create_pool_msg" $TXFLAG --from $deployer_address --amount=$amount | jq -r '.txhash')
	sleep $tx_delay
	local res=$($BINARY q tx $res --node $RPC -o json)

	local lp_asset=$(echo $res | jq -r '.events[] | select(.type == "wasm") | .attributes[] | select(.key == "lp_asset") | .value')
	local pool_type=$(echo $res | jq -r '.events[] | select(.type == "wasm") | .attributes[] | select(.key == "pool_type") | .value')
	pool_identifier=$(echo $res | jq -r '.events[] | select(.type == "wasm") | .attributes[] | select(.key == "pool_identifier") | .value')

	local label=$(echo "${label::-1}")

	denom_0="${asset_denoms[0]}"
	local decimal_0=${asset_decimals[0]}
	denom_1="${asset_denoms[1]}"
	local decimal_1=${asset_decimals[1]}

	# Store on output file
	tmpfile=$(mktemp)
	jq --arg label "$label" --arg pool_identifier "$pool_identifier" --arg denom_0 "$denom_0" --argjson decimal_0 "$decimal_0" --arg denom_1 "$denom_1" --argjson decimal_1 "$decimal_1" --arg pool_type "$pool_type" --arg lp_asset "$lp_asset" '.pools += [{label: $label, pool_identifier: $pool_identifier, assets: [{denom: $denom_0, decimals: $decimal_0}, {denom: $denom_1, decimals: $decimal_1}], pool_type: $pool_type, lp_asset: $lp_asset}]' $output_file >$tmpfile
	mv $tmpfile $output_file
	# Add additional deployment information
	date=$(date -u +"%Y-%m-%dT%H:%M:%S%z")
	tmpfile=$(mktemp)
	jq --arg date $date --arg chain_id $CHAIN_ID --arg pool_manager_addr $pool_manager_addr '. + {date: $date , chain_id: $chain_id, pool_manager_addr: $pool_manager_addr}' $output_file >$tmpfile
	mv $tmpfile $output_file

	echo -e "\n**** Created $label pool on $CHAIN_ID successfully ****\n"
	jq '.' $output_file
}

function provide_liquidity() {
	local provide_liquidity_msg='{"provide_liquidity":{"pool_identifier":"'$pool_identifier'"}}'

	if [ ${#amounts[@]} -ne 2 ]; then
		echo "You must provide liquidity for both assets"
		exit 1
	fi

	amount=${amounts[0]}$denom_0,${amounts[1]}$denom_1

	echo -e "\nProviding liquidity to:"
	echo "Pool: $pool_identifier"
	echo "Asset 0: ${amounts[0]}$denom_0"
	echo "Asset 1: ${amounts[1]}$denom_1"

	local res=$($BINARY tx wasm execute $pool_manager_addr "$provide_liquidity_msg" $TXFLAG --amount=$amount --from $deployer_address | jq -r '.txhash')
	sleep $tx_delay
	local res=$($BINARY q tx $res --node $RPC -o json)

	if [[ $(echo $res | jq -r '.raw_log') == "" ]]; then
		echo -e "\n**** Provided liquidity successfully ****\n"
	elif [[ $(echo $res | jq -r '.raw_log') == *"error"* ]]; then
		echo -e "\n**** Error providing liquidity ****\n"
		echo $res
	fi
}

if [ -z $1 ]; then
	display_usage
	exit 0
fi

# get args
optstring=':c:p:a:h'
while getopts $optstring arg; do
	case "$arg" in
	c)
		chain=$OPTARG
		source $deployment_script_dir/deploy_env/chain_env.sh
		init_chain_env $OPTARG
		if [[ "$chain" = "local" ]]; then
			tx_delay=0.5
		else
			tx_delay=8
		fi
		;;
	p)
		source $deployment_script_dir/wallet_importer.sh
		import_deployer_wallet $chain

		# read pool config from file $OPTARG
		read_pool_config $OPTARG && create_pool
		;;
	a)
		readarray -t amounts < <(awk -F',' '{ for( i=1; i<=NF; i++ ) print $i }' <<<"$OPTARG")
		provide_liquidity
		;;
	h)
		display_usage
		exit 0
		;;
	:)
		echo "Must supply an argument to -$OPTARG" >&2
		exit 1
		;;
	?)
		echo "Invalid option: -${OPTARG}"
		display_usage
		exit 2
		;;
	esac
done
