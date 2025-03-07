import { Log, TransactionReceipt, TransactionResponse, solidityPacked, AccessList } from "ethers";
import { addressOrZero } from "./utils";
import { EncodedFields } from "./common";

export function getFieldsForType0(tx: TransactionResponse): EncodedFields {
    return {
        types: [
            "uint8", "uint64", "uint128", "uint64", "address", "address", "uint256", "bytes", "uint256", "bytes32", "bytes32"
        ],
        values: [
            tx.type, tx.nonce, tx.gasPrice, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, tx.signature.networkV ?? tx.signature.v, tx.signature.r, tx.signature.s
        ]
    };
}

export function getFieldsForType1(tx: TransactionResponse): EncodedFields {
    return {
        types: [
            "uint8", "uint64", "uint64", "uint128", "uint64", "address", "address", "uint256", "bytes", "bytes[]", "uint8", "bytes32", "bytes32"
        ],
        values: [
            tx.type, tx.chainId, tx.nonce, tx.gasPrice, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, encodeAccessList(tx.accessList), tx.signature.yParity, tx.signature.r, tx.signature.s
        ]
    };
}

export function getFieldsForType2(tx: TransactionResponse): EncodedFields {
  return {
    types: [
      "uint8", "uint64", "uint64", "uint128", "uint128", "uint64", "address", "address", "uint256", "bytes", "bytes[]", "uint8", "bytes32", "bytes32"
    ],
    values: [
      tx.type, tx.chainId, tx.nonce, tx.maxPriorityFeePerGas ?? 0, tx.maxFeePerGas, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, encodeAccessList(tx.accessList), tx.signature.yParity, tx.signature.r, tx.signature.s
    ]
  };
}

export function getFieldsForType3(tx: TransactionResponse): EncodedFields {
  const out = {
    types: [
      "uint8", "uint64", "uint64", "uint128", "uint128", "uint64", "address", "address", "uint256", "bytes", "bytes[]", "uint128", "bytes32[]", "uint8", "bytes32", "bytes32"
    ],
    values: [
      tx.type, tx.chainId, tx.nonce, tx.maxPriorityFeePerGas ?? 0, tx.maxFeePerGas, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, encodeAccessList(tx.accessList), tx.maxFeePerBlobGas, tx.blobVersionedHashes, tx.signature.yParity, tx.signature.r, tx.signature.s
    ]
  };

  return out;
}

export function getFieldsForType(tx: TransactionResponse): EncodedFields {
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

export function encodeAccessListItem(item: { address: string, storageKeys: string[] }) {
    const accessListEncoded = solidityPacked(["address", "bytes32[]"], [item.address,item.storageKeys]);
    return accessListEncoded;
}

export function encodeAccessList(accessList: AccessList | null): string[] {

    if (!accessList)
        return [];

    const result = accessList.map(encodeAccessListItem);
    return result;
}

export function getReceiptFields(rx: TransactionReceipt): EncodedFields {
    return {
        types: [
            "uint8", "uint64", "bytes[]", "bytes"
        ],
        values: [
            rx.status, rx.gasUsed, rx.logs.map(encodeLog), rx.logsBloom
        ]
    };
}

export function encodeLog(log: Log): string {

    const result = solidityPacked(
        ["address", "bytes[]", "bytes"],
        [log.address, log.topics, log.data]
    )

    return result;
}

export function solidityPackedEncode(tx: TransactionResponse, rx: TransactionReceipt) {
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
