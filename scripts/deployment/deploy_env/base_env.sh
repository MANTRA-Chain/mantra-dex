#!/usr/bin/env bash

## Loads the base env for a given chain. i.e. the TXFLAG variable.

b_flag=sync
output_flag=json

case $chain in

mantra | mantra-testnet)
	if [ -n "$ZSH_VERSION" ]; then
		# Using an array for TXFLAG
		TXFLAG=(--node $RPC --chain-id $CHAIN_ID --gas-prices 0.3$DENOM --gas auto --gas-adjustment 1.2 -y -b $b_flag --output $output_flag)
	else
		# Using a string for TXFLAG
		TXFLAG="--node $RPC --chain-id $CHAIN_ID --gas-prices 0.3$DENOM --gas auto --gas-adjustment 1.2 -y -b $b_flag --output $output_flag"
	fi
	;;
*)
	echo "Network $chain not defined"
	return 1
	;;
esac

export TXFLAG
