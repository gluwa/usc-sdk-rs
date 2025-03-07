import { JsonRpcProvider } from "ethers";
import { abiEncode } from "./encodings/abi";
import { getBlockReceipts } from "./utils/block-receipt";
import { writeFileSync } from 'fs';
import { solidityPackedEncode } from "./encodings/packed-abi";
import { safeSolidityPackedEncode } from "./encodings/safe-packed-abi";

const rpc = "https://sepolia-proxy-rpc.creditcoin.network";
const provider = new JsonRpcProvider(rpc);

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
        //const safeSolidityPackedEncodedAbi = safeSolidityPackedEncode(tx, rx);


        encodedTransactions.push({
            transactionIndex: i,
            abi: encodedAbi,
            solidityPacked: solidityPackedEncodedAbi,
            // safeSolidityPacked: safeSolidityPackedEncodedAbi
        });
    }

    let end = performance.now();
    console.log('time to execute abi encoding of all kinds', end - start);

    const blockAbiFile = encodedTransactions.map(t => t.abi.abi);
    const packedBlockAbiFile = encodedTransactions.map(t => t.solidityPacked.abi);
    // const packedBlockAbiWithSeparatorFile = encodedTransactions.map(t => t.safeSolidityPacked.abi);
    writeToFile("../ignore/ethers-out/block.json", blockAbiFile);
    writeToFile("../ignore/ethers-out/solidity-packed-block.json", packedBlockAbiFile);
    // writeToFile("../ignore/ethers-out/safe-solidity-packed-block.json", packedBlockAbiWithSeparatorFile);
}

function writeToFile(file: string, data: any) {
    writeFileSync(file, JSON.stringify(data, null, 2), {
        encoding: 'utf-8'
    });
}

async function main() {
    // await loadBlockAndEncode(BigInt(7846292));
    await loadBlockAndEncode(BigInt(7853137));

    //let type_1 = "0x5c8c6d8c61bd8109ce02717db62b12554c097d156b66e30ff64864b5d4b1c041";
    //let type_3 = "0x085d2fe01372711005b053a1b0d081c13cde19b6ddb77cae847e0d11a0a0cafe";
    //let type_2 = "0xdfba59b94bac3da5af5d0fa8b81ae3199069fa6f38002be58c14e94a051e0642";
    //let legacy = "0x0b50111d729c00bac4a99702b2c88e425321c8f8214bc3272072c730d5ff9ad2";
    //let not_matching = "0xf09500718fa31ffb89bc0374b95f2b1f39047b2e3e01058984a9697e045a94b3";
    // let not_matching2 = "0xb044ddc49d105964890f8e197c85f42d23737356015a07586a4f9237666526a8";
    // await singleTransactionEncoding(not_matching2);
}

main()
    .then(() => console.log('yay'));