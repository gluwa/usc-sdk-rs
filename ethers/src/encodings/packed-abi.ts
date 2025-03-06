import { Log, TransactionReceipt, TransactionResponse, solidityPacked, AccessList } from "ethers";
import { addressOrZero } from "./utils";

interface EncodedFields {
    types: string[];
    values: any[];
}

function getFieldsForType0(tx: TransactionResponse): EncodedFields {
    return {
        types: [
            "uint256", "uint256", "uint256", "address", "address", "uint256", "bytes", "uint256", "bytes32", "bytes32"
        ],
        values: [
            tx.nonce, tx.gasPrice, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, tx.signature.v, tx.signature.r, tx.signature.s
        ]
    };
}

function getFieldsForType1(tx: TransactionResponse): EncodedFields {
    return {
        types: [
            "uint256", "uint256", "uint256", "uint256", "address", "address", "uint256", "bytes", "bytes[]", "uint256", "bytes32", "bytes32"
        ],
        values: [
            tx.chainId, tx.nonce, tx.gasPrice, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, encodeAccessList(tx.accessList), tx.signature.v, tx.signature.r, tx.signature.s
        ]
    };
}

function getFieldsForType2(tx: TransactionResponse): EncodedFields {
    return {
        types: [
            "uint256", "uint256", "uint256", "uint256", "uint256", "address", "address", "uint256", "bytes", "bytes[]", "uint256", "bytes32", "bytes32"
        ],
        values: [
            tx.chainId, tx.nonce, tx.maxPriorityFeePerGas, tx.maxFeePerGas, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, encodeAccessList(tx.accessList), tx.signature.v, tx.signature.r, tx.signature.s
        ]
    };
}

function getFieldsForType3(tx: TransactionResponse): EncodedFields {
  const out = {
    types: [
      "uint256", "uint256", "uint256", "uint256", "uint256", "address", "address", "uint256", "bytes", "bytes[]", "uint256", "bytes32[]", "uint256", "bytes32", "bytes32"
    ],
    values: [
      tx.chainId, tx.nonce, tx.maxPriorityFeePerGas, tx.maxFeePerGas, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, encodeAccessList(tx.accessList), tx.maxFeePerBlobGas, tx.blobVersionedHashes, tx.signature.v, tx.signature.r, tx.signature.s
    ]
  };

  return out;
}

function getFieldsForType(tx: TransactionResponse): EncodedFields {
    switch (tx.type) {
        case 0:
            return getFieldsForType0(tx);
        case 1:
            return getFieldsForType1(tx);
        case 2:
            return getFieldsForType2(tx);
        case 3:
            return getFieldsForType3(tx);
        default:
            throw new Error("Unsupported transaction type");
    }
}

function encodeAccessListItem(item: { address: string, storageKeys: string[] }) {
    const accessListEncoded = solidityPacked(["address", "bytes32[]"], [item.address,item.storageKeys]);
    return accessListEncoded;
}

function encodeAccessList(accessList: AccessList | null): string[] {

    if (!accessList)
        return [];

    const result = accessList.map(encodeAccessListItem);
    return result;
}

function getReceiptFields(rx: TransactionReceipt): EncodedFields {
    return {
        types: [
            "uint256", "uint256", "bytes[]", "bytes"
        ],
        values: [
            rx.status, rx.gasUsed, rx.logs.map(encodeLog), rx.logsBloom
        ]
    };
}

function encodeLog(log: Log): string {

    const result = solidityPacked(
        ["address", "bytes[]", "bytes"],
        [log.address, log.topics, log.data]
    )

    return result;
}

function abiEncode(tx: TransactionResponse, rx: TransactionReceipt) {
    const txFields = getFieldsForType(tx);
    const receiptFields = getReceiptFields(rx);
    const allFieldTypes = [...txFields.types, ...receiptFields.types];
    const allFieldValues = [...txFields.values, ...receiptFields.values];

    const abi = solidityPacked(allFieldTypes, allFieldValues);

    return {
        types: allFieldTypes,
        abi
    };
}

export { getFieldsForType, getReceiptFields, abiEncode };
