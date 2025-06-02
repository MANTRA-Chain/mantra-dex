const { SigningCosmWasmClient } = require('@cosmjs/cosmwasm-stargate');
const { LedgerSigner } = require('@cosmjs/ledger-amino');
const { makeCosmoshubPath } = require("@cosmjs/amino");
const TransportNodeHid = require('@ledgerhq/hw-transport-node-hid').default;
const { GasPrice } = require('@cosmjs/stargate');
const { toUtf8 } = require("@cosmjs/encoding");
const fs = require('fs');

const GAS_PRICE_STRING = "0.025uom";

async function main() {
    const rpcEndpoint = process.argv[2];
    const farmManagerAddress = process.argv[3];
    const accountIndex = parseInt(process.argv[4] || "0", 10);

    if (!rpcEndpoint || !farmManagerAddress) {
        console.error("Usage: node close_farms.js <rpc_endpoint> <farm_manager_address> [account_index]");
        console.error("Example: node close_farms.js \"http://localhost:26657\" \"mantra1...\" 0");
        process.exit(1);
    }

    console.log("Attempting to connect to Ledger device...");
    console.log("Please ensure your Ledger is connected, unlocked, and the Cosmos (or chain-specific) app is open.");

    let transport;
    try {
        transport = await TransportNodeHid.create();
    } catch (e) {
        console.error("Error connecting to Ledger device. Details:", e.message);
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
        await transport.close();
        process.exit(1);
    }

    if (accounts.length === 0) {
        console.error(`No accounts found on Ledger for HD Path m/44'/118'/0'/0/${accountIndex} with prefix 'mantra'.`);
        await transport.close();
        process.exit(1);
    }
    const senderAddress = accounts[0].address;

    const client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, ledgerSigner, {
        gasPrice: GasPrice.fromString(GAS_PRICE_STRING)
    });

    console.log(`Successfully connected to Ledger. Using address: ${senderAddress}`);
    console.log(`Connected to RPC endpoint: ${rpcEndpoint}`);
    console.log(`Targeting Farm Manager contract: ${farmManagerAddress}`);

    let allFarmIdentifiers = [];
    let startAfter = null;
    const limit = 100;

    try {
        console.log("Starting to query farms from the Farm Manager contract...");
        do {
            const queryMsg = {
                farms: { 
                    limit: limit,
                    ...(startAfter && { start_after: startAfter })
                }
            };
            
            console.log(`Querying with: ${JSON.stringify(queryMsg)}`);
            const farmsResponse = await client.queryContractSmart(farmManagerAddress, queryMsg);

            let farms = [];
            if (farmsResponse && Array.isArray(farmsResponse.farms)) {
                farms = farmsResponse.farms;
            } else {
                console.error("Unexpected response structure from 'farms' query:", farmsResponse);
                throw new Error("Failed to parse farm data. Expected a response with a 'farms' array.");
            }
            
            if (farms.length === 0) {
                if (startAfter === null) console.log("No farms found in the contract.");
                else console.log("Reached end of farm list.");
                break; 
            }

            let farmsInPage = 0;
            for (const farm of farms) {
                allFarmIdentifiers.push(farm.identifier);
                farmsInPage++;
            }
            console.log(`Fetched ${farms.length} farms in this page. Found ${farmsInPage} farms. Total so far: ${allFarmIdentifiers.length}`);

            if (farms.length === limit) {
                const lastFarm = farms[farms.length - 1];
                if (lastFarm && lastFarm.farm_identifier) {
                    startAfter = lastFarm.farm_identifier;
                } else {
                    console.warn("Last farm in page did not have farm_identifier, cannot paginate further this way.");
                    startAfter = null; 
                }
            } else {
                startAfter = null; // Last page fetched
            }
            
        } while (startAfter);

        console.log(`Finished querying. Total ${allFarmIdentifiers.length} farm(s) identifiers found.`);

        if (allFarmIdentifiers.length === 0) {
            console.log("No farms were found. No messages to generate. Exiting.");
            await transport.close();
            return;
        }

        const messages = allFarmIdentifiers.map(farmId => {
            const executeMsg = {
                manage_farm: {
                    action: {
                        close: {
                            farm_identifier: farmId
                        }
                    }
                }
            };
            return {
                typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
                value: {
                    sender: senderAddress,
                    contract: farmManagerAddress,
                    msg: toUtf8(JSON.stringify(executeMsg)),
                    funds: [],
                },
            };
        });

        console.log(`Generated ${messages.length} 'ManageFarm (Close)' execute messages for all farms.`);

        const outputFileName = `emergency_close_all_farms_tx_preview_${Date.now()}.json`;
        fs.writeFileSync(outputFileName, JSON.stringify({
            rpcEndpoint: rpcEndpoint,
            farmManagerAddress: farmManagerAddress,
            senderAddress: senderAddress,
            accountIndexUsed: accountIndex,
            hdPathUsed: `m/44'/118'/0'/0/${accountIndex}`,
            messagesCount: messages.length,
            generatedMessages: messages
        }, null, 2));
        console.log(`Transaction messages preview saved to: ${outputFileName}`);
        console.log("CRITICAL: Review this JSON file carefully before proceeding to sign and broadcast.");

        const readline = require('readline').createInterface({ input: process.stdin, output: process.stdout });
        const answer = await new Promise(resolve => {
            readline.question(`\nARE YOU ABSOLUTELY SURE you want to sign (with Ledger) and broadcast ${messages.length} messages to CLOSE ALL ${allFarmIdentifiers.length} farms on ${farmManagerAddress} via ${rpcEndpoint}? (yes/no): `, resolve);
        });
        readline.close();

        if (answer.toLowerCase() !== 'yes') {
            console.log("Operation cancelled by the user.");
            await transport.close();
            return;
        }

        console.log("Proceeding with signing and broadcasting... Please confirm on your Ledger device.");
        
        const gasPerMessage = 300000; // Estimate, adjust based on tx complexity and chain fees. The more farms to close, the more gas is needed.
        const totalGas = BigInt(messages.length * gasPerMessage);
        const feeGasPrice = GasPrice.fromString(GAS_PRICE_STRING);
        const calculatedFeeAmount = Math.ceil(Number(totalGas) * parseFloat(feeGasPrice.amount.toString())).toString();

        const fee = {
            amount: [{ denom: feeGasPrice.denom, amount: calculatedFeeAmount }],
            gas: totalGas.toString(),
        };
        const memo = "Emergency: Close all farms";
        console.log(`Calculated fee: ${calculatedFeeAmount}${feeGasPrice.denom} for ${totalGas.toString()} gas.`);

        try {
            const result = await client.signAndBroadcast(senderAddress, messages, fee, memo);
            if (result.code !== undefined && result.code !== 0) {
                console.error(`Transaction failed! Code: ${result.code}, Log: ${result.rawLog}`);
            } else {
                console.log(`Transaction broadcasted successfully! Hash: ${result.transactionHash}`);
            }
        } catch (broadcastError) {
             console.error("Error during signAndBroadcast:", broadcastError.message);
        }

    } catch (error) {
        console.error("An unexpected error occurred:", error.message);
        if (error.stack) console.error(error.stack);
    } finally {
        if (transport) {
            try { await transport.close(); console.log("Ledger transport closed."); }
            catch (closeError) { console.error("Error closing Ledger transport:", closeError.message); }
        }
    }
}

main().catch(error => {
    console.error("Critical script failure:", error.message);
    if (error.stack) console.error(error.stack);
    process.exit(1);
});
