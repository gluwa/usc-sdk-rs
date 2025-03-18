import { AbiCoder, hexlify, JsonRpcProvider, LogDescription, ParamType, ZeroAddress, Log } from "ethers";
import { abiEncode, getAllFields, getFieldsForType } from "./encodings/abi";
import { getBlockReceipts } from "./utils/block-receipt";
import { writeFileSync } from 'fs';
import { solidityPackedEncode } from "./encodings/packed-abi";
import { safeSolidityPackedEncode } from "./encodings/safe-packed-abi";
import { MappedEncodedFields, QueryableFields } from "./query-builder/abi/models";
import { computeAbiOffsets } from "./query-builder/abi/abi-utils";
import { play, play2 } from "./experimenting";
import { ForkedReader } from "./query-builder/common/ForkedReader";
import { QueryBuilder } from "./query-builder/abi/QueryBuilder";

const rpc = "https://sepolia-proxy-rpc.creditcoin.network";
const provider = new JsonRpcProvider(rpc);

const rpcMainnet = "https://mainnet-proxy-rpc.creditcoin.network";
const providerMainnet = new JsonRpcProvider(rpcMainnet);

async function singleTransactionEncoding(transactionHash: string) {
    //const transactionHash = "0xcdfb24b6f13f867c1f38d7263eb6f66b865a2e44203b52b068b2998df81200b1";
    const transaction = await provider.getTransaction(transactionHash);
    const receipt = await provider.getTransactionReceipt(transactionHash);

    // const abi = abiEncode(transaction!, receipt!);
    // console.log(JSON.stringify(abi));

    const solidityPackedEncoded = solidityPackedEncode(transaction!, receipt!);
    console.log(JSON.stringify(solidityPackedEncoded));

    // const safeSolidityPackedEncoded = safeSolidityPackedEncode(transaction!, receipt!);
    // console.log(JSON.stringify(safeSolidityPackedEncoded));
}

async function loadBlockAndEncode(blockNumber: bigint) {

    const start = performance.now()

    const block = await provider.getBlock(blockNumber, true);
    if (!block)
        throw new Error(`no block found with block number ${blockNumber}`);

    if (!block.hash)
        throw new Error('block has no hash.');

    const transactions = block.prefetchedTransactions;
    const receipts = await getBlockReceipts(provider, block.hash);
    let encodedTransactions = [];

    for (let i = 0; i < transactions.length; i++) {
        const tx = transactions[i];
        const rx = receipts[i];


        const encodedAbi = abiEncode(tx, rx);
        const solidityPackedEncodedAbi = solidityPackedEncode(tx, rx);

        let safeSolidityPackedEncodedAbi;
        try {
            safeSolidityPackedEncodedAbi = safeSolidityPackedEncode(tx, rx);
        }
        catch (ex) {
            console.error('failed to process', tx);
            throw ex;
        }


        encodedTransactions.push({
            transactionIndex: i,
            abi: encodedAbi,
            solidityPacked: solidityPackedEncodedAbi,
            safeSolidityPacked: safeSolidityPackedEncodedAbi
        });
    }

    let end = performance.now();
    console.log('time to execute abi encoding of all kinds', end - start);

    const blockAbiFile = encodedTransactions.map(t => t.abi.abi);
    const packedBlockAbiFile = encodedTransactions.map(t => t.solidityPacked.abi);
    const packedBlockAbiWithSeparatorFile = encodedTransactions.map(t => t.safeSolidityPacked.abi);
    writeToFile("../ignore/ethers-out/block.json", blockAbiFile);
    writeToFile("../ignore/ethers-out/solidity-packed-block.json", packedBlockAbiFile);
    writeToFile("../ignore/ethers-out/safe-solidity-packed-block.json", packedBlockAbiWithSeparatorFile);
}

function writeToFile(file: string, data: any) {
    writeFileSync(file, JSON.stringify(data, null, 2), {
        encoding: 'utf-8'
    });
}

async function experimentAbi() {

    const transactionHash = "0xdfba59b94bac3da5af5d0fa8b81ae3199069fa6f38002be58c14e94a051e0642"; //"0x0b50111d729c00bac4a99702b2c88e425321c8f8214bc3272072c730d5ff9ad2";
    const transaction = await provider.getTransaction(transactionHash);
    const receipt = await provider.getTransactionReceipt(transactionHash);

    const allFields = getAllFields(transaction!, receipt!);
    console.dir(allFields, { depth: null });
    const encoded = abiEncode(transaction!, receipt!);
    console.log('types', encoded.types, 'abi', encoded.abi);
    const paramTypes = encoded.types.map(type => ParamType.from(type));
    console.log('parsed param types');
    const computedOffsets = computeAbiOffsets(paramTypes, encoded.abi);
    console.dir(computedOffsets, { depth: null });

    // const logs = computedOffsets.find(t => t.type == "(address, bytes32[], bytes)[]")!;
    // const log = logs.children[0];
    // const log0AddressPositionOffset = log.children[0].offset;
    // const log0AddressPositionSize = log.children[0].size;
    // const reader = new ForkedReader(encoded.abi);
    // reader.jumpTo(log0AddressPositionOffset);
    // const log0addressValue = hexlify(reader.readBytes(log0AddressPositionSize!));
    // console.log(log0addressValue);

    // const txFields = getFieldsForType(transaction!);
    // console.dir(txFields, { depth: null });
    // const encoded = AbiCoder.defaultAbiCoder().encode(txFields.types, txFields.values);
    // console.log('types', txFields.types, 'abi', encoded);
    // const paramTypes = txFields.types.map(type => ParamType.from(type));  
    // console.log('parsed param types');  
    // const computedOffsets = computeAbiOffsets(paramTypes, encoded);
    // console.dir(computedOffsets, { depth: null });
}

async function experimentQueryBuilder() {
    const transactionHash = "0xc990ce703dd3ca83429c302118f197651678de359c271f205b9083d4aa333aae"; //"0x0b50111d729c00bac4a99702b2c88e425321c8f8214bc3272072c730d5ff9ad2";
    const transaction = await provider.getTransaction(transactionHash);
    const receipt = await provider.getTransactionReceipt(transactionHash);
    const builder = QueryBuilder.createFromTransaction(transaction!, receipt!);
    const abiEncoded = abiEncode(transaction!, receipt!);
    console.log('ABI', abiEncoded.abi);


    //const computedOffsets = computeAbiOffsets(abiEncoded.types.map(type => ParamType.from(type)), abiEncoded.abi);
    // console.dir(computedOffsets, { depth: 5 });
    // return;


    // if you're wondering this is the contract of GCRE on testnet sepolia.
    builder.setAbiProvider(async (contractAddress) => {
        return `
            [{"constant":false,"inputs":[{"name":"tokenHolders","type":"address[]"},{"name":"amounts","type":"uint256[]"}],"name":"recordSales730Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[],"name":"VestingStartDate","outputs":[{"name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"name","outputs":[{"name":"","type":"string"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"spender","type":"address"},{"name":"value","type":"uint256"}],"name":"approve","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"totalSupply","outputs":[{"name":"amount","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf365Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"value","type":"uint256"},{"name":"sighash","type":"string"}],"name":"exchange","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"from","type":"address"},{"name":"to","type":"address"},{"name":"value","type":"uint256"}],"name":"transferFrom","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf183Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"decimals","outputs":[{"name":"","type":"uint8"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolders","type":"address[]"},{"name":"amounts","type":"uint256[]"}],"name":"recordSales1095Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf365Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"value","type":"uint256"}],"name":"burn","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf2190Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolders","type":"address[]"},{"name":"amounts","type":"uint256[]"}],"name":"recordSales183Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolder","type":"address"},{"name":"numCoins","type":"uint256"}],"name":"recordSale365Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf730Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"owner","type":"address"}],"name":"balanceOf","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[],"name":"finalizeSales","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"from","type":"address"},{"name":"value","type":"uint256"}],"name":"burnFrom","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf2190Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf730Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"symbol","outputs":[{"name":"","type":"string"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolder","type":"address"},{"name":"numCoins","type":"uint256"}],"name":"recordSale183Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"to","type":"address"},{"name":"value","type":"uint256"}],"name":"transfer","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[],"name":"creditcoinSalesLimit","outputs":[{"name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf1095Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"creditcoinLimitInFrac","outputs":[{"name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolder","type":"address"},{"name":"numCoins","type":"uint256"}],"name":"recordSale2190Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolder","type":"address"},{"name":"numCoins","type":"uint256"}],"name":"recordSale730Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf1095Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"owner","type":"address"},{"name":"spender","type":"address"}],"name":"allowance","outputs":[{"name":"remaining","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[],"name":"startVesting","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolders","type":"address[]"},{"name":"amounts","type":"uint256[]"}],"name":"recordSales2190Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf183Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"IsSalesFinalized","outputs":[{"name":"","type":"bool"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolders","type":"address[]"},{"name":"amounts","type":"uint256[]"}],"name":"recordSales365Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolder","type":"address"},{"name":"numCoins","type":"uint256"}],"name":"recordSale1095Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"inputs":[{"name":"creditcoinFoundation","type":"address"},{"name":"devCost","type":"address"}],"payable":false,"stateMutability":"nonpayable","type":"constructor"},{"payable":true,"stateMutability":"payable","type":"fallback"},{"anonymous":false,"inputs":[{"indexed":true,"name":"from","type":"address"},{"indexed":false,"name":"value","type":"uint256"},{"indexed":true,"name":"sighash","type":"string"}],"name":"Exchange","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"from","type":"address"},{"indexed":false,"name":"value","type":"uint256"}],"name":"Burnt","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"from","type":"address"},{"indexed":true,"name":"to","type":"address"},{"indexed":false,"name":"value","type":"uint256"}],"name":"Transfer","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"owner","type":"address"},{"indexed":true,"name":"spender","type":"address"},{"indexed":false,"name":"value","type":"uint256"}],"name":"Approval","type":"event"}]
        `;
    });

    builder
        .addStaticField(QueryableFields.TxFrom)
        .addStaticField(QueryableFields.TxTo)
        .addStaticField(QueryableFields.TxNonce)
        .addStaticField(QueryableFields.RxStatus);

    // the reason this one checks for a matching topic
    // even though its not a requirement if the ABI dosen't have two different
    // kind of transfer Events, but it kind of makes it safer :)
    const burnTransferFilter = (log: Log, logDescription: LogDescription, _: number) => {

        if (logDescription.topic != "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef")
            return false;

        if (log.address.toLowerCase() != "0x47C30768E4c153B40d55b90F58472bb2291971e6".toLowerCase())
            return false;

        return logDescription.args.from.toLowerCase() == "0x9d6bC9763008AD1F7619a3498EfFE9Ec671b276D".toLowerCase() && logDescription.args.to.toLowerCase() == ZeroAddress.toLowerCase();
    };

    // notice this filter function dosen't require a check of the topic
    // thats because we are using add event argument by signature
    // which ensures no other type of events can be loaded.
    const burntEventFilter = (log: Log, logDescription: LogDescription, _: number) => {
        if (log.address.toLowerCase() != "0x47C30768E4c153B40d55b90F58472bb2291971e6".toLowerCase())
            return false;

        return logDescription.args.from.toLowerCase() == "0x9d6bC9763008AD1F7619a3498EfFE9Ec671b276D".toLowerCase();
    };

    // more optimical way, and more fluent.
    await builder.eventBuilder("0x919f7e2092ffcc9d09f599be18d8152860b0c054df788a33bc549cdd9d0f15b1", burntEventFilter, b => b
        .addSignature().addArgument("from").addArgument("value")
    );

    await builder.eventBuilder("Transfer", burnTransferFilter, b => b
        .addSignature().addArgument("from").addArgument("to").addArgument("value")
    );

    // less optimal way, burnt event field.
    // await builder.addEventSignature("0x919f7e2092ffcc9d09f599be18d8152860b0c054df788a33bc549cdd9d0f15b1", burntEventFilter);
    // await builder.addEventArgument("0x919f7e2092ffcc9d09f599be18d8152860b0c054df788a33bc549cdd9d0f15b1", "from", burntEventFilter);
    // await builder.addEventArgument("0x919f7e2092ffcc9d09f599be18d8152860b0c054df788a33bc549cdd9d0f15b1", "value", burntEventFilter);

    // tranfer event fields.
    // await builder.addEventSignature("Transfer", burnTransferFilter);
    // await builder.addEventArgument("Transfer", "from", burnTransferFilter);
    // await builder.addEventArgument("Transfer", "to", burnTransferFilter);
    // await builder.addEventArgument("Transfer", "value", burnTransferFilter); 

    // toying with function arguments :)
    // you can ether use the function name or the function signature :D
    await builder.functionBuilder("burn", b => b
        .addSignature().addArgument("value")
    );

    // builder.addFunctionSignature();
    // await builder.addFunctionArgument("burn", "value");
    // await builder.addFunctionArgument("0x42966c68", "value"); 


    const fields = builder.build();

    const reader = new ForkedReader(abiEncoded.abi);
    fields.forEach(field => {
        reader.jumpTo(field.offset);
        const data = reader.readBytes(field.size);
        console.log(field, '\t', hexlify(data));
    });
}

async function complexMultiEventScenario() {
    const transactionHash = "0xd746058082c13f30e2e219f78a150e536ccacf71b262e6fa5ce0f4b4a17a1cf6";
    const transaction = await providerMainnet.getTransaction(transactionHash);
    const receipt = await providerMainnet.getTransactionReceipt(transactionHash);
    const builder = QueryBuilder.createFromTransaction(transaction!, receipt!);
    const abiEncoded = abiEncode(transaction!, receipt!);

    // if you're wondering this is the contract of GCRE on testnet sepolia.
    builder.setAbiProvider(async (contractAddress) => {
        return `[{"constant":false,"inputs":[{"name":"tokenHolders","type":"address[]"},{"name":"amounts","type":"uint256[]"}],"name":"recordSales730Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[],"name":"VestingStartDate","outputs":[{"name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"name","outputs":[{"name":"","type":"string"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"spender","type":"address"},{"name":"value","type":"uint256"}],"name":"approve","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"totalSupply","outputs":[{"name":"amount","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf365Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"value","type":"uint256"},{"name":"sighash","type":"string"}],"name":"exchange","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"from","type":"address"},{"name":"to","type":"address"},{"name":"value","type":"uint256"}],"name":"transferFrom","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf183Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"decimals","outputs":[{"name":"","type":"uint8"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolders","type":"address[]"},{"name":"amounts","type":"uint256[]"}],"name":"recordSales1095Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf365Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"value","type":"uint256"}],"name":"burn","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf2190Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolders","type":"address[]"},{"name":"amounts","type":"uint256[]"}],"name":"recordSales183Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolder","type":"address"},{"name":"numCoins","type":"uint256"}],"name":"recordSale365Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf730Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"owner","type":"address"}],"name":"balanceOf","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[],"name":"finalizeSales","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"from","type":"address"},{"name":"value","type":"uint256"}],"name":"burnFrom","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf2190Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf730Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"symbol","outputs":[{"name":"","type":"string"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolder","type":"address"},{"name":"numCoins","type":"uint256"}],"name":"recordSale183Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"to","type":"address"},{"name":"value","type":"uint256"}],"name":"transfer","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[],"name":"creditcoinSalesLimit","outputs":[{"name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf1095Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"creditcoinLimitInFrac","outputs":[{"name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolder","type":"address"},{"name":"numCoins","type":"uint256"}],"name":"recordSale2190Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolder","type":"address"},{"name":"numCoins","type":"uint256"}],"name":"recordSale730Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf1095Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"owner","type":"address"},{"name":"spender","type":"address"}],"name":"allowance","outputs":[{"name":"remaining","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[],"name":"startVesting","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolders","type":"address[]"},{"name":"amounts","type":"uint256[]"}],"name":"recordSales2190Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf183Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"IsSalesFinalized","outputs":[{"name":"","type":"bool"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolders","type":"address[]"},{"name":"amounts","type":"uint256[]"}],"name":"recordSales365Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolder","type":"address"},{"name":"numCoins","type":"uint256"}],"name":"recordSale1095Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"inputs":[{"name":"creditcoinFoundation","type":"address"},{"name":"devCost","type":"address"}],"payable":false,"stateMutability":"nonpayable","type":"constructor"},{"payable":true,"stateMutability":"payable","type":"fallback"},{"anonymous":false,"inputs":[{"indexed":true,"name":"from","type":"address"},{"indexed":false,"name":"value","type":"uint256"},{"indexed":true,"name":"sighash","type":"string"}],"name":"Exchange","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"from","type":"address"},{"indexed":false,"name":"value","type":"uint256"}],"name":"Burnt","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"from","type":"address"},{"indexed":true,"name":"to","type":"address"},{"indexed":false,"name":"value","type":"uint256"}],"name":"Transfer","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"owner","type":"address"},{"indexed":true,"name":"spender","type":"address"},{"indexed":false,"name":"value","type":"uint256"}],"name":"Approval","type":"event"}]`;
    });

    builder
        .addStaticField(QueryableFields.RxStatus);

    await builder.multiEventBuilder("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef", (log, logDescription, index) => true, b => b
        .addAddress()
        .addSignature()
        .addArgument("from")
        .addArgument("to")
        .addArgument("value")
    );

    const fields = builder.build();
    const reader = new ForkedReader(abiEncoded.abi);
    fields.forEach(field => {
        reader.jumpTo(field.offset);
        const data = reader.readBytes(field.size);
        console.log(field, '\t', hexlify(data));
    });

    // this query can be use as such in solidity.
    // statusField = resultSegments[0];
    // assert(statusField == 1)
    // next it can do
    // uint256 quantityOfTransferEvents = (resultSegments.length - 1)/5;
    // loop through
    // slice(i * 5, 5);
    // ensure that the first element is indeed the contract of the token you expect.
    // assert(resultSegments[i*5] == 0x000000000000000000000000a3ee21c306a700e682abcdfe9baa6a08f3820419);
    // etc..
}

async function main() {

    //await experimentAbi();
    await play2();
    //await experimentQueryBuilder();
    //await complexMultiEventScenario();

    // await loadBlockAndEncode(BigInt(7846292));
    // await loadBlockAndEncode(BigInt(7853137));

    //let type_1 = "0x5c8c6d8c61bd8109ce02717db62b12554c097d156b66e30ff64864b5d4b1c041";
    //let type_3 = "0x085d2fe01372711005b053a1b0d081c13cde19b6ddb77cae847e0d11a0a0cafe";
    //let type_2 = "0xdfba59b94bac3da5af5d0fa8b81ae3199069fa6f38002be58c14e94a051e0642";
    //let legacy = "0x0b50111d729c00bac4a99702b2c88e425321c8f8214bc3272072c730d5ff9ad2";
    //let not_matching = "0xf09500718fa31ffb89bc0374b95f2b1f39047b2e3e01058984a9697e045a94b3";
    // let not_matching2 = "0xb044ddc49d105964890f8e197c85f42d23737356015a07586a4f9237666526a8";
    // await singleTransactionEncoding(not_matching2);
}

main()
    .then(() => { });