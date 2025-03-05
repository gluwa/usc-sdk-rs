import { ethers } from "ethers";

interface EncodedFields {
    types: string[];
    values: any[];
}

function getFieldsForType0(tx: ethers.providers.TransactionResponse): EncodedFields {
    return {
        types: [
            "uint256", "uint256", "uint256", "address", "address", "uint256", "bytes", "uint256", "bytes32", "bytes32"
        ],
        values: [
            tx.nonce, tx.gasPrice, tx.gasLimit, tx.from, tx.to, tx.value, tx.data, tx.v, tx.r, tx.s
        ]
    };
}

function getFieldsForType1(tx: ethers.providers.TransactionResponse): EncodedFields {
    return {
        types: [
            "uint256", "uint256", "uint256", "uint256", "address", "address", "uint256", "bytes", "bytes[]", "uint256", "bytes32", "bytes32"
        ],
        values: [
            tx.chainId, tx.nonce, tx.gasPrice, tx.gasLimit, tx.from, tx.to, tx.value, tx.data, encodeAccessList(tx.accessList), tx.v, tx.r, tx.s
        ]
    };
}

function getFieldsForType2(tx: ethers.providers.TransactionResponse): EncodedFields {
    return {
        types: [
            "uint256", "uint256", "uint256", "uint256", "uint256", "address", "address", "uint256", "bytes", "bytes[]", "uint256", "bytes32", "bytes32"
        ],
        values: [
            tx.chainId, tx.nonce, tx.maxPriorityFeePerGas, tx.maxFeePerGas, tx.gasLimit, tx.from, tx.to, tx.value, tx.data, encodeAccessList(tx.accessList), tx.v, tx.r, tx.s
        ]
    };
}

function getFieldsForType(tx: ethers.providers.TransactionResponse): EncodedFields {
    switch (tx.type) {
        case 0:
            return getFieldsForType0(tx);
        case 1:
            return getFieldsForType1(tx);
        case 2:
            return getFieldsForType2(tx);
        default:
            throw new Error("Unsupported transaction type");
    }
}

function encodeAccessListItem(item: { address: string, storageKeys: string[] }) {
    const accessListEncoded = ethers.utils.solidityPack(["address", "uint256[]"], [item.address,item.storageKeys]);
    return accessListEncoded;
}

function encodeAccessList(accessList: ethers.utils.AccessList | undefined): string[] {

    if (!accessList)
        return [];

    const result = accessList.map(encodeAccessListItem);
    return result;
}

function getReceiptFields(rx: ethers.providers.TransactionReceipt): EncodedFields {
    return {
        types: [
            "uint256", "uint256", "bytes[]", "bytes"
        ],
        values: [
            rx.status, rx.gasUsed, rx.logs.map(encodeLog), rx.logsBloom
        ]
    };
}

function encodeLog(log: ethers.providers.Log): string {

    const result = ethers.utils.solidityPack(
        ["address", "bytes[]", "bytes"],
        [log.address, log.topics, log.data]
    )

    return result;
}

function abiEncode(tx: ethers.providers.TransactionResponse, rx: ethers.providers.TransactionReceipt) {
    const txFields = getFieldsForType(tx);
    const receiptFields = getReceiptFields(rx);
    const allFieldTypes = [...txFields.types, ...receiptFields.types];
    const allFieldValues = [...txFields.values, ...receiptFields.values];

    const abi = ethers.utils.solidityPack(allFieldTypes, allFieldValues);

    return {
        types: allFieldTypes,
        abi
    };
}

export { getFieldsForType, getReceiptFields, abiEncode };
