import { Log, TransactionReceipt, TransactionResponse, solidityPacked, AccessList } from "ethers";

interface EncodedFields {
    types: string[];
    values: any[];
}

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


function getFieldsForType0(tx: TransactionResponse): EncodedFields {
    const out = {
        types: [
            "uint256", "uint256", "uint256", "address", "address", "uint256", "bytes", "uint256", "bytes32", "bytes32"
        ],
        values: [
            tx.nonce, tx.gasPrice, tx.gasLimit, tx.from, tx.to, tx.value, tx.data, tx.signature.v, tx.signature.r, tx.signature.s
        ]
    };
    const safe = insertSeparator(out);
    return safe;
}

function getFieldsForType1(tx: TransactionResponse): EncodedFields {
    const out = {
        types: [
            "uint256", "uint256", "uint256", "uint256", "address", "address", "uint256", "bytes", "bytes[]", "uint256", "bytes32", "bytes32"
        ],
        values: [
            tx.chainId, tx.nonce, tx.gasPrice, tx.gasLimit, tx.from, tx.to, tx.value, tx.data, encodeAccessList(tx.accessList), tx.signature.v, tx.signature.r, tx.signature.s
        ]
    };
    const safe = insertSeparator(out);
    return safe;
}

function getFieldsForType2(tx: TransactionResponse): EncodedFields {
    const out = {
        types: [
            "uint256", "uint256", "uint256", "uint256", "uint256", "address", "address", "uint256", "bytes", "bytes[]", "uint256", "bytes32", "bytes32"
        ],
        values: [
            tx.chainId, tx.nonce, tx.maxPriorityFeePerGas, tx.maxFeePerGas, tx.gasLimit, tx.from, tx.to, tx.value, tx.data, encodeAccessList(tx.accessList), tx.signature.v, tx.signature.r, tx.signature.s
        ]
    };
    const safe = insertSeparator(out);
    return safe;
}

function getFieldsForType3(tx: TransactionResponse): EncodedFields {
  const out = {
    types: [
      "uint256", "uint256", "uint256", "uint256", "uint256", "address", "uint256", "bytes", "bytes[]", "uint256", "bytes32[]", "uint256", "bytes32", "bytes32"
    ],
    values: [
      tx.chainId, tx.nonce, tx.maxPriorityFeePerGas, tx.maxFeePerGas, tx.gasLimit, tx.to, tx.value, tx.data, encodeAccessList(tx.accessList), tx.maxFeePerBlobGas, tx.blobVersionedHashes, tx.signature.v, tx.signature.r, tx.signature.s
    ]
  };

  const safe = insertSeparator(out);
  return safe;
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

    const fields: EncodedFields = {
        types: ["address", "uint256[]"],
        values: [item.address,item.storageKeys]
    };

    const safe = insertSeparator(fields);
    const accessListEncoded = solidityPacked(safe.types, safe.values);
    return accessListEncoded;
}

function encodeAccessList(accessList: AccessList | null): string[] {

    if (!accessList)
        return [];

    const result = accessList.map(encodeAccessListItem);
    return result;
}

function getReceiptFields(rx: TransactionReceipt): EncodedFields {
    const fields = {
        types: [
            "uint256", "uint256", "bytes[]", "bytes"
        ],
        values: [
            rx.status, rx.gasUsed, rx.logs.map(encodeLog), rx.logsBloom
        ]
    };

    const safe = insertSeparator(fields);
    return safe;
}

function encodeLog(log: Log): string {

    const logFields: EncodedFields = {
        types:  ["address", "bytes[]", "bytes"],
        values: [log.address, log.topics, log.data]
    }

    const safe = insertSeparator(logFields);
    const result = solidityPacked(safe.types, safe.values);
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
