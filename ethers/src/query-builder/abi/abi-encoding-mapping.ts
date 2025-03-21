import { MappedEncodedFields, QueryableFields } from "./models";

function getMappedFieldsForType0(): MappedEncodedFields {
    return {
        fields: [
            { name: QueryableFields.Type, type: "uint8" },
            { name: QueryableFields.TxNonce, type: "uint64" },
            { name: QueryableFields.TxGasPrice, type: "uint128" },
            { name: QueryableFields.TxGasLimit, type: "uint64" },
            { name: QueryableFields.TxFrom, type: "address" },
            { name: QueryableFields.TxTo, type: "address" },
            { name: QueryableFields.TxValue, type: "uint256" },
            { name: QueryableFields.TxData, type: "bytes" },
            { name: QueryableFields.TxV, type: "uint256" },
            { name: QueryableFields.TxR, type: "bytes32" },
            { name: QueryableFields.TxS, type: "bytes32" }
        ]
    };
}

function getMappedFieldsForType1(): MappedEncodedFields {
    return {
        fields: [
            { name: QueryableFields.Type, type: "uint8" },
            { name: QueryableFields.TxChainId, type: "uint64" },
            { name: QueryableFields.TxNonce, type: "uint64" },
            { name: QueryableFields.TxGasPrice, type: "uint128" },
            { name: QueryableFields.TxGasLimit, type: "uint64" },
            { name: QueryableFields.TxFrom, type: "address" },
            { name: QueryableFields.TxTo, type: "address" },
            { name: QueryableFields.TxValue, type: "uint256" },
            { name: QueryableFields.TxData, type: "bytes" },
            { name: QueryableFields.TxAccessList, type: "tuple(address,bytes32[])[]" },
            { name: QueryableFields.TxYParity, type: "uint8" },
            { name: QueryableFields.TxR, type: "bytes32" },
            { name: QueryableFields.TxS, type: "bytes32" }
        ]
    };
}

function getMappedFieldsForType2(): MappedEncodedFields {
    return {
        fields: [
            { name: QueryableFields.Type, type: "uint8" },
            { name: QueryableFields.TxChainId, type: "uint64" },
            { name: QueryableFields.TxNonce, type: "uint64" },
            { name: QueryableFields.TxMaxPriorityFeePerGas, type: "uint128" },
            { name: QueryableFields.TxMaxFeePerGas, type: "uint128" },
            { name: QueryableFields.TxGasLimit, type: "uint64" },
            { name: QueryableFields.TxFrom, type: "address" },
            { name: QueryableFields.TxTo, type: "address" },
            { name: QueryableFields.TxValue, type: "uint256" },
            { name: QueryableFields.TxData, type: "bytes" },
            { name: QueryableFields.TxAccessList, type: "tuple(address,bytes32[])[]" },
            { name: QueryableFields.TxYParity, type: "uint8" },
            { name: QueryableFields.TxR, type: "bytes32" },
            { name: QueryableFields.TxS, type: "bytes32" }
        ]
    };
}

function getMappedFieldsForType3(): MappedEncodedFields {
    return {
        fields: [
            { name: QueryableFields.Type, type: "uint8" },
            { name: QueryableFields.TxChainId, type: "uint64" },
            { name: QueryableFields.TxNonce, type: "uint64" },
            { name: QueryableFields.TxMaxPriorityFeePerGas, type: "uint128" },
            { name: QueryableFields.TxMaxFeePerGas, type: "uint128" },
            { name: QueryableFields.TxGasLimit, type: "uint64" },
            { name: QueryableFields.TxFrom, type: "address" },
            { name: QueryableFields.TxTo, type: "address" },
            { name: QueryableFields.TxValue, type: "uint256" },
            { name: QueryableFields.TxData, type: "bytes" },
            { name: QueryableFields.TxAccessList, type: "tuple(address,uint256[])[]" },
            { name: QueryableFields.TxMaxFeePerBlobGas, type: "uint256" },
            { name: QueryableFields.TxBlobVersionedHashes, type: "bytes32[]" },
            { name: QueryableFields.TxYParity, type: "uint8" },
            { name: QueryableFields.TxR, type: "bytes32" },
            { name: QueryableFields.TxS, type: "bytes32" }
        ]
    };
}

function getMappedFieldsForType(type: number): MappedEncodedFields {
    switch (type) {
        case 0:
            return getMappedFieldsForType0();
        case 1:
            return getMappedFieldsForType1();
        case 2:
            return getMappedFieldsForType2();
        case 3:
            return getMappedFieldsForType3();
        default:
            throw new Error("Unsupported transaction type");
    }
}

function getMappedReceiptFields(): MappedEncodedFields {
    return {
        fields: [
            { name: QueryableFields.RxStatus, type: "uint8" },
            { name: QueryableFields.RxGasUsed, type: "uint64" },
            { name: QueryableFields.RxLogs, type: "tuple(address, bytes32[], bytes)[]" },
            { name: QueryableFields.RxLogBlooms, type: "bytes" }
        ]
    };
}

export function getAllFieldsForTransaction(type: number): MappedEncodedFields {
    const txFields = getMappedFieldsForType(type);
    const rxFields = getMappedReceiptFields();
    const allFields: MappedEncodedFields = {
        fields: [...txFields.fields, ...rxFields.fields]
    }
    return allFields;
}