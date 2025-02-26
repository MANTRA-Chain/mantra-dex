const path = require('path');
const fs = require('fs');
const {toUtf8} = require("@cosmjs/encoding");
const readline = require('readline');
const {DirectSecp256k1HdWallet} = require('@cosmjs/proto-signing');
const {SigningCosmWasmClient} = require('@cosmjs/cosmwasm-stargate');

require('dotenv').config();

let chainId;
let denom;
let binary;
let rpc;
let mnemonic;
let poolManagerAddr;
let poolData;

const POOL_CREATION_AND_TF_FEE = {denom: 'uom', amount: '176000000'};

async function main() {
    const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {prefix: 'mantra'});
    const [{address}] = await wallet.getAccounts();
    const client = await SigningCosmWasmClient.connectWithSigner(rpc, wallet);

    const protocol_fee = poolData.protocol_fee;
    const swap_fee = poolData.swap_fee;
    const burn_fee = poolData.burn_fee;
    const pool_type = poolData.pool_type;
    const pool_identifier = poolData.pool_identifier;
    const amp_factor = pool_type === 'stable_swap' ? Number(poolData.amp_factor) : null;

    const assetDenoms = poolData.assets.map(asset => asset.denom);
    const assetDecimals = poolData.assets.map(asset => asset.decimals);

    const poolType = pool_type === 'stable_swap'
        ? {"stable_swap": {"amp": amp_factor}}
        : "constant_product";

    const createPoolMsg = {
        typeUrl: '/cosmwasm.wasm.v1.MsgExecuteContract',
        value: {
            sender: address,
            contract: poolManagerAddr,
            msg: toUtf8(JSON.stringify({
                create_pool: {
                    asset_denoms: assetDenoms,
                    asset_decimals: assetDecimals,
                    pool_fees: {
                        protocol_fee: {share: protocol_fee},
                        swap_fee: {share: swap_fee},
                        burn_fee: {share: burn_fee},
                        extra_fees: []
                    },
                    pool_type: poolType,
                    pool_identifier: pool_identifier
                }
            })),
            funds: [POOL_CREATION_AND_TF_FEE]
        }
    };

    // Read amounts from command-line arguments
    const amount0 = process.argv[4];
    const amount1 = process.argv[5];

    let funds = [
        {denom: assetDenoms[0], amount: amount0},
        {denom: assetDenoms[1], amount: amount1}
    ];
    funds = sortFunds(funds);

    const provideLiquidityMsg = {
        typeUrl: '/cosmwasm.wasm.v1.MsgExecuteContract',
        value: {
            sender: address,
            contract: poolManagerAddr,
            msg: toUtf8(JSON.stringify({
                provide_liquidity: {
                    pool_identifier: `o.${pool_identifier}`
                }
            })),
            funds: funds
        }
    };

    const fee = {
        amount: [{amount: '1200000', denom: 'uom'}],
        gas: '1200000',
    };

    // Adjust the messages as per the contract requirements
    const msgs = [createPoolMsg, provideLiquidityMsg];

    // Display confirmation message
    console.log('\n⚠️ WARNING ⚠️️');
    console.log('\nCreating pool with the following configuration:');
    console.log(`Chain id: ${chainId}`);
    console.log(`Asset 0: ${assetDenoms[0]} - decimals: ${assetDecimals[0]}`);
    console.log(`Asset 1: ${assetDenoms[1]} - decimals: ${assetDecimals[1]}`);
    console.log(`Pool type: ${pool_type}`);
    if (pool_type === 'stable_swap') {
        console.log(`Amp factor: ${amp_factor}`);
    }
    console.log(`Pool identifier: ${pool_identifier}`);
    console.log(`Protocol fee: ${protocol_fee}`);
    console.log(`Swap fee: ${swap_fee}`);
    console.log(`Burn fee: ${burn_fee}`);
    console.log(`Funds: ${JSON.stringify(funds)}`);

    // Prompt user for confirmation
    const proceed = await promptUser('\nDo you want to proceed? (y/n) ');

    if (proceed.toLowerCase() !== 'y') {
        console.log('Pool deploymendt cancelled...');
        process.exit(1);
    }

    try {
        const result = await client.signAndBroadcast(address, msgs, fee);
        if (result.code !== 0) {
            throw new Error(`Transaction failed with code ${result.code}: ${result.rawLog || 'No details available'}`);
        }
        console.log('Transaction succeeded with hash:', result.transactionHash)

        const label = `${assetDenoms[0].split('/').pop()}-${assetDenoms[1].split('/').pop()}`;
        const lp_asset = `factory/${poolManagerAddr}/o.${pool_identifier}.LP`;
        await updatePoolOutputFile({
            label,
            pool_identifier: `o.${pool_identifier}`,
            assets: [
                {denom: assetDenoms[0], decimals: assetDecimals[0]},
                {denom: assetDenoms[1], decimals: assetDecimals[1]}
            ],
            pool_type: pool_type === 'stable_swap' ? 'StableSwap' : 'ConstantProduct',
            lp_asset
        });

        console.log(`\n**** Created ${label} pool on ${chainId} successfully ****\n`);
        console.log('Updated pool data:', JSON.stringify(JSON.parse(fs.readFileSync(getOutputFilePath())), null, 2));
    } catch (error) {
        console.error('Transaction failed:', error);
    }
}


function readChainInfo(env) {
    let envPath;
    if (env === 'mantra') {
        envPath = path.join(__dirname, 'deploy_env', 'mainnets', 'mantra.env');
    } else if (env === 'mantra-testnet') {
        envPath = path.join(__dirname, 'deploy_env', 'testnets', 'mantra.env');
    } else {
        console.error('Invalid environment specified. Use "mantra" or "mantra-testnet".');
        return;
    }

    require('dotenv').config({path: envPath});

    chainId = process.env.CHAIN_ID;
    denom = process.env.DENOM;
    binary = process.env.BINARY;
    rpc = process.env.RPC;
}

function readPoolConfig(poolJsonFilePath) {
    if (fs.existsSync(poolJsonFilePath)) {
        const rawData = fs.readFileSync(poolJsonFilePath);
        poolData = JSON.parse(rawData);
    } else {
        console.error('JSON file not found:', poolJsonFilePath);
    }
}

function readMnemonicFromFile(env) {
    let mnemonicFilePath;
    if (env === 'mantra') {
        mnemonicFilePath = path.join(__dirname, 'deploy_env', 'mnemonics', 'deployer_mnemonic.txt');
    } else if (env === 'mantra-testnet') {
        mnemonicFilePath = path.join(__dirname, 'deploy_env', 'mnemonics', 'deployer_mnemonic_testnet.txt');
    } else {
        console.error('Invalid environment specified. Use "mantra" or "mantra-testnet".');
        return;
    }

    try {
        mnemonic = fs.readFileSync(mnemonicFilePath, 'utf8').trim();
    } catch (error) {
        console.error('Error reading mnemonic file:', error);
        process.exit(1);
    }
}

function readPoolManagerAddr(env) {
    let outputFilePath;
    if (env === 'mantra') {
        outputFilePath = path.join(__dirname, 'output', 'mantra-1_mantra_dex_contracts.json');
    } else if (env === 'mantra-testnet') {
        outputFilePath = path.join(__dirname, 'output', 'mantra-dukong-1_mantra_dex_contracts.json');
    } else {
        console.error('Invalid environment specified. Use "mantra" or "mantra-testnet".');
        return;
    }

    try {
        const data = fs.readFileSync(outputFilePath, 'utf8').trim();
        let json = JSON.parse(data);

        const contract = json.contracts.find(contract => contract.wasm === 'pool_manager.wasm');
        if (contract && contract.contract_address) {
            poolManagerAddr = contract.contract_address;
        } else {
            console.error(`Contract address not found for pool_manager.wasm`);
            process.exit(1);
        }
    } catch (error) {
        console.error('Error reading deployment file:', error);
        process.exit(1);
    }
}

function sortFunds(funds) {
    return funds.sort((a, b) => a.denom.localeCompare(b.denom));
}

// Function to prompt the user for confirmation
function promptUser(question) {
    const rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout
    });

    return new Promise((resolve) => {
        rl.question(question, (answer) => {
            rl.close();
            resolve(answer);
        });
    });
}

function getOutputFilePath() {
    return path.join(__dirname, 'output', `${chainId}_pools.json`);
}

async function updatePoolOutputFile(newPool) {
    const outputFilePath = getOutputFilePath();
    let outputData = {pools: [], date: '', chain_id: '', pool_manager_addr: ''};

    // Read existing file if it exists
    if (fs.existsSync(outputFilePath)) {
        outputData = JSON.parse(fs.readFileSync(outputFilePath, 'utf8'));
    }

    // Add new pool entry
    outputData.pools.push(newPool);

    // Update metadata
    const now = new Date().toISOString().replace('Z', '+0000');
    outputData.date = now;
    outputData.chain_id = chainId;
    outputData.pool_manager_addr = poolManagerAddr;

    // Write back to file
    fs.writeFileSync(outputFilePath, JSON.stringify(outputData, null, 2));
}


const env = process.argv[2];
const poolJsonFilePath = process.argv[3];
readChainInfo(env);
readPoolConfig(poolJsonFilePath);
readMnemonicFromFile(env);
readPoolManagerAddr(env);

main().catch(console.error);
