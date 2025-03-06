import { JsonRpcProvider  } from "ethers";

export async function getBlockReceipts(rpc: JsonRpcProvider, blockHash: string) {
    const receiptsRaw: Array<any> = await rpc.send('eth_getBlockReceipts', [blockHash]);
    const receipts = receiptsRaw.map(r => {
        const receipt = rpc._wrapTransactionReceipt(r, rpc._network);
        return receipt;
    });
    return receipts;
}