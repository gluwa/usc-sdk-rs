import { ethers } from "ethers";
import { abiEncode } from "./encodings/abi";
import { abiEncode as packedAbiEncode } from "./encodings/packed-abi";

async function main() {
    const rpc = "https://sepolia-proxy-rpc.creditcoin.network";
    const transactionHash = "0xe3de4394fc39316c737abe75768a0050d69cb610956434d7cd7d8bb0fa7d5b90";
    const provider = new ethers.providers.JsonRpcProvider(rpc);
    const transaction = await provider.getTransaction(transactionHash);
    const receipt = await provider.getTransactionReceipt(transactionHash);
    // const abi = abiEncode(transaction, receipt);
    // console.log(JSON.stringify(abi));

    
    const packedAbi = packedAbiEncode(transaction, receipt);
    console.log(JSON.stringify(packedAbi));
}

main()
    .then(() => console.log('yay'));