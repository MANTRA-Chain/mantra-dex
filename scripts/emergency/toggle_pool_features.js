const { SigningCosmWasmClient } = require('@cosmjs/cosmwasm-stargate');
const { LedgerSigner } = require('@cosmjs/ledger-amino');
const { makeCosmoshubPath } = require("@cosmjs/amino"); // makeCosmoshubPath is often in @cosmjs/amino
const TransportNodeHid = require('@ledgerhq/hw-transport-node-hid').default;
const { GasPrice } = require('@cosmjs/stargate');
const { toUtf8 } = require("@cosmjs/encoding");
const fs = require('fs');

const GAS_PRICE_STRING = "0.025uom"; 

async function main() {
    const rpcEndpoint = process.argv[2];
    const poolManagerAddress = process.argv[3];
    const accountIndex = parseInt(process.argv[4] || "0", 10); // Optional account index, defaults to 0

    if (!rpcEndpoint || !poolManagerAddress) {
        console.error("Usage: node toggle_pool_features.js <rpc_endpoint> <pool_manager_address> [account_index]");
        console.error("Example: node toggle_pool_features.js \"http://localhost:26657\" \"mantra1...\" 0");
        process.exit(1);
    }

    console.log("Attempting to connect to Ledger device...");
    console.log("Please ensure your Ledger is connected, unlocked, and the Cosmos (or chain-specific) app is open.");

    let transport;
    try {
        transport = await TransportNodeHid.create();
    } catch (e) {
        console.error("Error connecting to Ledger device. Details:", e.message);
        console.error("Troubleshooting: \n1. Is the Ledger plugged in and unlocked?\n2. Is the correct Ledger app (e.g., Cosmos, Mantra) open?\n3. Is another application (e.g., Ledger Live) using the Ledger? Close it.\n4. (Linux users) Are udev rules set up correctly for Ledger devices?");
        process.exit(1);
    }

    const ledgerSigner = new LedgerSigner(transport, {
        hdPaths: [makeCosmoshubPath(accountIndex)],
        prefix: 'mantra'
    });

    let accounts;
    try {
        accounts = await ledgerSigner.getAccounts();
    } catch (e) {
        console.error("Error getting accounts from Ledger. Details:", e.message);
        console.error("Ensure the correct app is open on Ledger and it's not locked.");
        await transport.close();
        process.exit(1);
    }

    if (accounts.length === 0) {
        console.error("No accounts found on Ledger for the specified HD path and prefix.");
        console.error(`HD Path: m/44'/118'/0'/0/${accountIndex}, Prefix: mantra`);
        console.error("Please check your Ledger setup and the app.");
        await transport.close();
        process.exit(1);
    }
    const senderAddress = accounts[0].address;

    const client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, ledgerSigner, {
        gasPrice: GasPrice.fromString(GAS_PRICE_STRING)
    });

    console.log(`Successfully connected to Ledger. Using address: ${senderAddress}`);
    console.log(`Connected to RPC endpoint: ${rpcEndpoint}`);
    console.log(`Targeting Pool Manager contract: ${poolManagerAddress}`);

    let allPoolIdentifiers = [];
    let startAfter = null;
    const limit = 100;

    try {
        console.log("Starting to query all pool identifiers from the pool manager contract...");
        do {
            const queryMsg = {
                pools: {
                    limit: limit,
                    ...(startAfter && { start_after: startAfter })
                }
            };
            
            console.log(`Querying with: ${JSON.stringify(queryMsg)}`);
            const poolsResponse = await client.queryContractSmart(poolManagerAddress, queryMsg);

            if (poolsResponse && Array.isArray(poolsResponse.pools)) {
                const pools = poolsResponse.pools;
                
                if (pools.length === 0 && startAfter === null) {
                    console.log("No pools found in the contract.");
                    break; 
                }
                if (pools.length === 0 && startAfter !== null) { 
                     console.log("Reached end of pool list.");
                     break;
                }

                for (const pool of pools) {
                    if (pool.pool_info.pool_identifier) { 
                        allPoolIdentifiers.push(pool.pool_info.pool_identifier);
                    } else {
                        console.warn("Found pool data entry without a 'pool_identifier' field:", pool);
                    }
                }
                console.log(`Fetched ${pools.length} pools in this page. Total fetched so far: ${allPoolIdentifiers.length}`);

                if (pools.length === limit) {
                    // Get the pool_identifier of the last fetched pool for pagination
                    const lastPool = pools[pools.length - 1];
                   if (lastPool?.pool_info?.pool_identifier) {
                        startAfter = lastPool.pool_info.pool_identifier;
                    } else {
                        console.warn("Last pool in page did not have pool_identifier, cannot paginate further this way.");
                        startAfter = null; // Stop pagination
                    }
                } else {
                    startAfter = null; // Last page fetched
                }
            } else {
                console.error("Unexpected response structure from 'all_pools_info' query:", poolsResponse);
                throw new Error("Failed to parse pool data. Expected a response with a 'pools' array.");
            }
            
        } while (startAfter);

        console.log(`Finished querying. Total ${allPoolIdentifiers.length} pool(s) identifiers found.`);

        if (allPoolIdentifiers.length === 0) {
            console.log("No pools were found in the manager contract. No messages to generate. Exiting.");
            await transport.close();
            return;
        }

        const messages = allPoolIdentifiers.map(poolId => {
            const executeMsg = {
                update_config: {
                    feature_toggle: {
                        pool_identifier: poolId,
                        withdrawals_enabled: false,
                        deposits_enabled: false,
                        swaps_enabled: false
                    }
                }
            };
            return {
                typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
                value: {
                    sender: senderAddress,
                    contract: poolManagerAddress,
                    msg: toUtf8(JSON.stringify(executeMsg)),
                    funds: [],
                },
            };
        });

        console.log(`Generated ${messages.length} 'UpdateConfig' execute messages to toggle OFF features for all pools.`);

        const outputFileName = `emergency_toggle_off_features_tx_preview_${Date.now()}.json`;
        fs.writeFileSync(outputFileName, JSON.stringify({
            rpcEndpoint: rpcEndpoint,
            poolManagerAddress: poolManagerAddress,
            senderAddress: senderAddress,
            accountIndexUsed: accountIndex,
            hdPathUsed: `m/44'/118'/0'/0/${accountIndex}`,
            messagesCount: messages.length,
            generatedMessages: messages
        }, null, 2));
        console.log(`Transaction messages preview saved to: ${outputFileName}`);
        console.log("CRITICAL: Review this JSON file carefully before proceeding to sign and broadcast.");

        // Confirmation step
        const readline = require('readline').createInterface({
            input: process.stdin,
            output: process.stdout
        });

        const answer = await new Promise(resolve => {
            readline.question(`\nARE YOU ABSOLUTELY SURE you want to sign (with Ledger) and broadcast ${messages.length} messages to TOGGLE OFF (disable withdrawals, deposits, swaps) features for ALL ${allPoolIdentifiers.length} pools via the Pool Manager at ${poolManagerAddress} on ${rpcEndpoint}? (yes/no): `, resolve);
        });
        readline.close();

        if (answer.toLowerCase() !== 'yes') {
            console.log("Operation cancelled by the user.");
            await transport.close();
            return;
        }

        console.log("Proceeding with signing and broadcasting...");
        console.log("Please confirm each message (or the transaction bundle) on your Ledger device.");
        
        const gasPerMessage = 300000; // Estimate, adjust based on tx complexity and chain fees. The more pools to toggle, the more gas is needed.
        const totalGas = BigInt(messages.length * gasPerMessage);
        const feeGasPrice = GasPrice.fromString(GAS_PRICE_STRING);
        const calculatedFeeAmount = Math.ceil(Number(totalGas) * parseFloat(feeGasPrice.amount.toString())).toString();

        const fee = {
            amount: [{
                denom: feeGasPrice.denom,
                amount: calculatedFeeAmount,
            }],
            gas: totalGas.toString(),
        };
        const memo = "Emergency: Toggle OFF all pool features (disable deposits/withdrawals/swaps)";

        console.log(`Calculated fee: ${calculatedFeeAmount}${feeGasPrice.denom} for ${totalGas.toString()} gas.`);

        try {
            const result = await client.signAndBroadcast(senderAddress, messages, fee, memo);

            if (result.code !== undefined && result.code !== 0) {
                console.error(`Transaction failed! Code: ${result.code}, Log: ${result.rawLog}`);
                console.error("Check the raw log for contract-specific errors.");
            } else {
                console.log(`Transaction broadcasted successfully! Hash: ${result.transactionHash}`);
                console.log("Please verify the transaction on a block explorer using the hash.");
            }
        } catch (broadcastError) {
             console.error("Error during signAndBroadcast:", broadcastError.message);
             if (broadcastError.message.includes("Ledger")) {
                 console.error("This might be a Ledger communication issue or rejection on the device.");
             }
        }

    } catch (error) {
        console.error("An unexpected error occurred during the script execution:", error.message);
        if (error.stack) {
            console.error(error.stack);
        }
        if (error.message && (error.message.includes("Ledger") || (error.code && error.code.toString(16).includes("6a80")) || (error.code && error.code.toString(16).includes("6985")) ) ) {
            console.error("A Ledger-related error was detected. Common issues:");
            console.error("- Ledger device disconnected or locked.");
            console.error("- Incorrect Ledger app open (must be Cosmos/Mantra app).");
            console.error("- User rejected the transaction on the Ledger device (0x6985).");
            console.error("- Invalid data sent to Ledger (0x6a80) - check message format.");
        }
    } finally {
        if (transport) {
            try {
                await transport.close();
                console.log("Ledger transport closed.");
            } catch (closeError) {
                console.error("Error closing Ledger transport:", closeError.message);
            }
        }
    }
}

main().catch(error => {
    console.error("Critical script failure:", error.message);
    if (error.stack) {
        console.error(error.stack);
    }
});
