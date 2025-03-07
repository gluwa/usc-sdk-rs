import { Log, TransactionReceipt, TransactionResponse, solidityPacked, AccessList } from "ethers";
import { addressOrZero } from "./utils";
import { EncodedFields } from "./common";


const SEPARATOR = 0; // uint8 zero separator

function insertSeparator(fields: EncodedFields): EncodedFields {
    const newTypes: string[] = [];
    const newValues: any[] = [];

    for (let i = 0; i < fields.types.length; i++) {
        // Add the original type
        newTypes.push(fields.types[i]);
        newValues.push(fields.values[i]);

        // Add separator unless it's the last element
        if (i < fields.types.length - 1) {
            newTypes.push("uint8"); // Separator type
            newValues.push(SEPARATOR); // uint8(0) as separator
        }
    }

    return { types: newTypes, values: newValues };
}


export function safeGetFieldsForType0(tx: TransactionResponse): EncodedFields {

    const out = {
        types: [
            "uint8", "uint64", "uint128", "uint64", "address", "address", "uint256", "bytes", "uint256", "bytes32", "bytes32"
        ],
        values: [
            tx.type, tx.nonce, tx.gasPrice, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, tx.signature.networkV ?? tx.signature.v, tx.signature.r, tx.signature.s
        ]
    };

    const safe = insertSeparator(out);
    return safe;
}

export function safeGetFieldsForType1(tx: TransactionResponse): EncodedFields {
    const out = {
        types: [
            "uint8", "uint64", "uint64", "uint128", "uint64", "address", "address", "uint256", "bytes", "bytes[]", "uint8", "bytes32", "bytes32"
        ],
        values: [
            tx.type, tx.chainId, tx.nonce, tx.gasPrice, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, safeEncodeAccessList(tx.accessList), tx.signature.yParity, tx.signature.r, tx.signature.s
        ]
    };
    const safe = insertSeparator(out);
    return safe;
}

export function safeGetFieldsForType2(tx: TransactionResponse): EncodedFields {
    const out = {
        types: [
            "uint8", "uint64", "uint64", "uint128", "uint128", "uint64", "address", "address", "uint256", "bytes", "bytes[]", "uint8", "bytes32", "bytes32"
        ],
        values: [
            tx.type, tx.chainId, tx.nonce, tx.maxPriorityFeePerGas ?? 0, tx.maxFeePerGas, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, safeEncodeAccessList(tx.accessList), tx.signature.yParity, tx.signature.r, tx.signature.s
        ]
    };
    const safe = insertSeparator(out);
    return safe;
}

export function safeGetFieldsForType3(tx: TransactionResponse): EncodedFields {
    const out = {
        types: [
            "uint8", "uint64", "uint64", "uint128", "uint128", "uint64", "address", "address", "uint256", "bytes", "bytes[]", "uint128", "bytes32[]", "uint8", "bytes32", "bytes32"
        ],
        values: [
            tx.type, tx.chainId, tx.nonce, tx.maxPriorityFeePerGas ?? 0, tx.maxFeePerGas, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, safeEncodeAccessList(tx.accessList), tx.maxFeePerBlobGas, tx.blobVersionedHashes, tx.signature.yParity, tx.signature.r, tx.signature.s
        ]
    };

    const safe = insertSeparator(out);
    return safe;
}

export function safeGetFieldsForType(tx: TransactionResponse): EncodedFields {
    switch (tx.type) {
        case 0: return safeGetFieldsForType0(tx);
        case 1: return safeGetFieldsForType1(tx);
        case 2: return safeGetFieldsForType2(tx);
        case 3: return safeGetFieldsForType3(tx);
        default:
            throw new Error("Unsupported transaction type");
    }
}

function safeEncodeAccessListItem(item: { address: string, storageKeys: string[] }) {

    const fields: EncodedFields = {
        types: ["address", "bytes32[]"],
        values: [item.address, item.storageKeys]
    };

    const safe = insertSeparator(fields);
    const accessListEncoded = solidityPacked(safe.types, safe.values);
    return accessListEncoded;
}

function safeEncodeAccessList(accessList: AccessList | null): string[] {

    if (!accessList)
        return [];

    const result = accessList.map(safeEncodeAccessListItem);
    return result;
}

function safeGetReceiptFields(rx: TransactionReceipt): EncodedFields {
    const fields = {
        types: [
            "uint8", "uint64", "bytes[]", "bytes"
        ],
        values: [
            rx.status, rx.gasUsed, rx.logs.map(safeEncodeLog), rx.logsBloom
        ]
    };

    const safe = insertSeparator(fields);
    return safe;
}

function safeEncodeLog(log: Log): string {

    const logFields: EncodedFields = {
        types: ["address", "bytes[]", "bytes"],
        values: [log.address, log.topics, log.data]
    }

    const safe = insertSeparator(logFields);
    const result = solidityPacked(safe.types, safe.values);
    return result;
}

export function safeSolidityPackedEncode(tx: TransactionResponse, rx: TransactionReceipt) {
    const txFields = safeGetFieldsForType(tx);
    const receiptFields = safeGetReceiptFields(rx);
    const allFieldTypes = [...txFields.types, ...['uint8'], ...receiptFields.types];
    const allFieldValues = [...txFields.values, ...[SEPARATOR], ...receiptFields.values];

    const abi = solidityPacked(allFieldTypes, allFieldValues);

    return {
        types: allFieldTypes,
        abi
    };
}
