import { JsonRpcProvider } from "ethers";
import { abiEncode } from "./encodings/abi";
import { abiEncode as packedAbiEncode } from "./encodings/packed-abi";
import { abiEncode as packedAbiEncodeWithSeparator } from "./encodings/packed-abi-seperated";
import { getBlockReceipts } from "./utils/block-receipt";
import { writeFileSync } from 'fs';

const rpc = "https://sepolia-proxy-rpc.creditcoin.network";
const provider = new JsonRpcProvider(rpc);

async function singleTransactionEncoding(transactionHash: string) {
    //const transactionHash = "0xcdfb24b6f13f867c1f38d7263eb6f66b865a2e44203b52b068b2998df81200b1";
    const transaction = await provider.getTransaction(transactionHash);
    const receipt = await provider.getTransactionReceipt(transactionHash);

    const abi = abiEncode(transaction!, receipt!);
    console.log(JSON.stringify(abi));

    // const packedAbi = packedAbiEncode(transaction!, receipt!);
    // console.log(JSON.stringify(packedAbi));

    // const packedAbiWithSeperator = packedAbiEncodeWithSeparator(transaction!, receipt!);
    // console.log(JSON.stringify(packedAbiWithSeperator));
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

    for(let i = 0; i < transactions.length; i++) {
        const tx = transactions[i];
        const rx = receipts[i];
        

        const encodedAbi = abiEncode(tx, rx);
        const packedAbiEncoded = packedAbiEncode(tx, rx);
        const packedAbiEncodedWithSeparator = packedAbiEncodeWithSeparator(tx, rx);


        encodedTransactions.push({
            transactionIndex: i,
            abi: encodedAbi,
            packedAbiEncode: packedAbiEncoded,
            packedAbiEncodedWithSeparator: packedAbiEncodedWithSeparator
        });
    }

    let end = performance.now();
    console.log('time to execute abi encoding of all kinds', end-start);
    
    const blockAbiFile = encodedTransactions.map(t => t.abi.abi);
    const packedBlockAbiFile = encodedTransactions.map(t => t.packedAbiEncode.abi);
    const packedBlockAbiWithSeparatorFile = encodedTransactions.map(t => t.packedAbiEncodedWithSeparator.abi);
    writeToFile("../ignore/ethers-out/block.json", blockAbiFile);
    writeToFile("../ignore/ethers-out/packed-block.json", packedBlockAbiFile);
    writeToFile("../ignore/ethers-out/packed-safe-block.json", packedBlockAbiWithSeparatorFile);
}

function writeToFile(file: string, data: any) {
    writeFileSync(file, JSON.stringify(data), {
        encoding: 'utf-8'
    });
}

async function main() {
    //await loadBlockAndEncode(BigInt(7846292));
    await singleTransactionEncoding("0xdfba59b94bac3da5af5d0fa8b81ae3199069fa6f38002be58c14e94a051e0642"); //"0x0b50111d729c00bac4a99702b2c88e425321c8f8214bc3272072c730d5ff9ad2");
}

main()
    .then(() => console.log('yay'));